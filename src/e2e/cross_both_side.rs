#[cfg(test)]
pub mod e2e_tests {
    use crate::InvariantError;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
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
        MAX_SQRT_PRICE, MIN_SQRT_PRICE,
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, create_dex, create_pool, create_position, create_tokens,
        get_pool, get_tick, mint, swap,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_cross_both_side(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let alice = ink_e2e::alice();
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

        let mint_amount = 10u128.pow(5);
        mint!(
            client,
            TokenRef,
            token_x,
            address_of!(Bob),
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

        let liquidity_delta = Liquidity::from_integer(20006000);

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

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            -20,
            lower_tick_index,
            liquidity_delta,
            pool_state.sqrt_price,
            pool_state.sqrt_price,
            alice
        )
        .unwrap();

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        assert_eq!(pool.liquidity, liquidity_delta);

        let limit_without_cross_tick_amount = TokenAmount(10_068);
        let not_cross_amount = TokenAmount(1);
        let min_amount_to_cross_from_tick_price = TokenAmount(3);
        let crossing_amount_by_amount_out = TokenAmount(20136101434);

        let mint_amount = limit_without_cross_tick_amount.get()
            + not_cross_amount.get()
            + min_amount_to_cross_from_tick_price.get()
            + crossing_amount_by_amount_out.get();

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

        let pool_before = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let limit_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            limit_without_cross_tick_amount,
            true,
            limit_sqrt_price,
            alice
        )
        .unwrap();

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        let expected_tick = -10;
        let expected_price = calculate_sqrt_price(expected_tick).unwrap();

        assert_eq!(pool.current_tick_index, expected_tick);
        assert_eq!(pool.liquidity, pool_before.liquidity);
        assert_eq!(pool.sqrt_price, expected_price);

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            min_amount_to_cross_from_tick_price,
            true,
            limit_sqrt_price,
            alice
        )
        .unwrap();

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            false,
            min_amount_to_cross_from_tick_price,
            true,
            SqrtPrice::new(MAX_SQRT_PRICE),
            alice
        )
        .unwrap();

        let massive_x = 10u128.pow(19);
        let massive_y = 10u128.pow(19);

        mint!(
            client,
            TokenRef,
            token_x,
            address_of!(Alice),
            massive_x,
            alice
        )
        .unwrap();
        mint!(
            client,
            TokenRef,
            token_y,
            address_of!(Alice),
            massive_y,
            alice
        )
        .unwrap();
        approve!(client, TokenRef, token_x, dex, massive_x, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, massive_y, alice).unwrap();

        let massive_liquidity_delta = Liquidity::from_integer(19996000399699881985603u128);

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            -20,
            0,
            massive_liquidity_delta,
            SqrtPrice::new(MIN_SQRT_PRICE),
            SqrtPrice::new(MAX_SQRT_PRICE),
            alice
        )
        .unwrap();

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            TokenAmount(1),
            false,
            limit_sqrt_price,
            alice
        )
        .unwrap();

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            false,
            TokenAmount(2),
            true,
            SqrtPrice::new(MAX_SQRT_PRICE),
            alice
        )
        .unwrap();

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let expected_liquidity = Liquidity::from_integer(19996000399699901991603u128);
        let expected_liquidity_change_on_last_tick =
            Liquidity::from_integer(19996000399699901991603u128);
        let expected_liquidity_change_on_upper_tick = Liquidity::from_integer(20006000);

        assert_eq!(pool.current_tick_index, -20);
        assert_eq!(
            pool.fee_growth_global_x,
            FeeGrowth::new(29991002699190242927121)
        );
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
        assert_eq!(pool.fee_protocol_token_x, TokenAmount(4));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount(2));
        assert_eq!(pool.liquidity, expected_liquidity);
        assert_eq!(pool.sqrt_price, SqrtPrice::new(999500149964999999999999));

        let final_last_tick = get_tick!(client, InvariantRef, dex, pool_key, -20).unwrap();
        assert_eq!(final_last_tick.fee_growth_outside_x, FeeGrowth::new(0));
        assert_eq!(final_last_tick.fee_growth_outside_y, FeeGrowth::new(0));
        assert_eq!(
            final_last_tick.liquidity_change,
            expected_liquidity_change_on_last_tick
        );

        let final_lower_tick = get_tick!(client, InvariantRef, dex, pool_key, -10).unwrap();
        assert_eq!(
            final_lower_tick.fee_growth_outside_x,
            FeeGrowth::new(29991002699190242927121)
        );
        assert_eq!(final_lower_tick.fee_growth_outside_y, FeeGrowth::new(0));
        assert_eq!(final_lower_tick.liquidity_change, Liquidity::new(0));

        let final_upper_tick = get_tick!(client, InvariantRef, dex, pool_key, 10).unwrap();
        assert_eq!(final_upper_tick.fee_growth_outside_x, FeeGrowth::new(0));
        assert_eq!(final_upper_tick.fee_growth_outside_y, FeeGrowth::new(0));
        assert_eq!(
            final_upper_tick.liquidity_change,
            expected_liquidity_change_on_upper_tick
        );

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_cross_both_side_not_cross_case(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let alice = ink_e2e::alice();
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

        let mint_amount = 10u128.pow(5);
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

        let liquidity_delta = Liquidity::new(20006000000000);

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

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            -20,
            lower_tick_index,
            liquidity_delta,
            pool_state.sqrt_price,
            pool_state.sqrt_price,
            alice
        )
        .unwrap();

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        assert_eq!(pool.liquidity, liquidity_delta);

        let limit_without_cross_tick_amount = TokenAmount(10_068);
        let not_cross_amount = TokenAmount(1);
        let min_amount_to_cross_from_tick_price = TokenAmount(3);
        let crossing_amount_by_amount_out = TokenAmount(20136101434);

        let mint_amount = limit_without_cross_tick_amount.get()
            + not_cross_amount.get()
            + min_amount_to_cross_from_tick_price.get()
            + crossing_amount_by_amount_out.get();

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

        let pool_before = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let limit_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            limit_without_cross_tick_amount,
            true,
            limit_sqrt_price,
            alice
        )
        .unwrap();

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        let expected_tick = -10;
        let expected_price = calculate_sqrt_price(expected_tick).unwrap();

        assert_eq!(pool.current_tick_index, expected_tick);
        assert_eq!(pool.liquidity, pool_before.liquidity);
        assert_eq!(pool.sqrt_price, expected_price);

        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);

        let result = swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            not_cross_amount,
            true,
            slippage,
            alice
        );

        assert_eq!(result, Err(InvariantError::NoGainSwap));
        Ok(())
    }
}
