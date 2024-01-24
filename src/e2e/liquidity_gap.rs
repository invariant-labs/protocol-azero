#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
        InvariantError,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::{
        types::{
            fee_growth::FeeGrowth,
            liquidity::Liquidity,
            percentage::Percentage,
            sqrt_price::{calculate_sqrt_price, SqrtPrice},
            token_amount::TokenAmount,
        },
        MIN_SQRT_PRICE,
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, mint, quote, swap,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_liquidity_gap(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let alice = ink_e2e::alice();
        let bob = ink_e2e::bob();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let initial_mint = 10u128.pow(10);

        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(1, 2));
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_mint, initial_mint);

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        let lower_tick_index = -10;
        let upper_tick_index = 10;

        let mint_amount = 10u128.pow(10);
        mint!(
            client,
            TokenRef,
            token_x,
            address_of!(Alice),
            mint_amount,
            alice
        )
        .unwrap();
        mint!(
            client,
            TokenRef,
            token_y,
            address_of!(Alice),
            mint_amount,
            alice
        )
        .unwrap();

        approve!(client, TokenRef, token_x, dex, mint_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, mint_amount, alice).unwrap();

        let liquidity_delta = Liquidity::from_integer(20_006_000);

        let pool_state = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            pool_state.sqrt_price,
            pool_state.sqrt_price,
            alice
        )
        .unwrap();

        let pool_state = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        assert_eq!(pool_state.liquidity, liquidity_delta);

        let mint_amount = 10067;
        mint!(
            client,
            TokenRef,
            token_x,
            address_of!(Bob),
            mint_amount,
            alice
        )
        .unwrap();

        approve!(client, TokenRef, token_x, dex, mint_amount, bob).unwrap();

        let dex_x_before = balance_of!(client, TokenRef, token_x, dex);
        let dex_y_before = balance_of!(client, TokenRef, token_y, dex);

        let swap_amount = TokenAmount::new(10067);
        let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);
        let quoted_target_sqrt_price = quote!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            target_sqrt_price
        )
        .unwrap()
        .target_sqrt_price;

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            quoted_target_sqrt_price,
            bob
        )
        .unwrap();

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        let expected_price = calculate_sqrt_price(-10).unwrap();
        let expected_y_amount_out = 9999;

        assert_eq!(pool.liquidity, liquidity_delta);
        assert_eq!(pool.current_tick_index, lower_tick_index);
        assert_eq!(pool.sqrt_price, expected_price);

        let bob_x = balance_of!(client, TokenRef, token_x, address_of!(Bob));
        let bob_y = balance_of!(client, TokenRef, token_y, address_of!(Bob));
        let dex_x_after = balance_of!(client, TokenRef, token_x, dex);
        let dex_y_after = balance_of!(client, TokenRef, token_y, dex);

        let delta_dex_x = dex_x_after - dex_x_before;
        let delta_dex_y = dex_y_before - dex_y_after;

        assert_eq!(bob_x, 0);
        assert_eq!(bob_y, expected_y_amount_out);
        assert_eq!(delta_dex_x, swap_amount.get());
        assert_eq!(delta_dex_y, expected_y_amount_out);
        assert_eq!(
            pool.fee_growth_global_x,
            FeeGrowth::new(29991002699190242927121)
        );
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(1));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(0));

        // No gain swap
        {
            let swap_amount = TokenAmount(1);
            let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

            let result = swap!(
                client,
                InvariantRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                target_sqrt_price,
                bob
            );

            assert_eq!(result, Err(InvariantError::NoGainSwap));
        }

        // Should skip gap and then swap
        let lower_tick_after_swap = -90;
        let upper_tick_after_swap = -50;
        let liquidity_delta = Liquidity::from_integer(20008000);

        approve!(client, TokenRef, token_x, dex, liquidity_delta.get(), alice).unwrap();
        approve!(client, TokenRef, token_y, dex, liquidity_delta.get(), alice).unwrap();

        let pool_state = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            lower_tick_after_swap,
            upper_tick_after_swap,
            liquidity_delta,
            pool_state.sqrt_price,
            pool_state.sqrt_price,
            alice
        )
        .unwrap();

        let swap_amount = TokenAmount::new(5000);
        mint!(
            client,
            TokenRef,
            token_x,
            address_of!(Bob),
            swap_amount.get(),
            alice
        )
        .unwrap();

        approve!(client, TokenRef, token_x, dex, swap_amount.get(), bob).unwrap();

        let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);
        let quoted_target_sqrt_price = quote!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            target_sqrt_price
        )
        .unwrap()
        .target_sqrt_price;

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            quoted_target_sqrt_price,
            bob
        )
        .unwrap();
        get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        Ok(())
    }
}
