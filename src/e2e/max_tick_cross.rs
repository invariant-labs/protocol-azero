#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::{
        log::get_tick_at_sqrt_price,
        types::{
            liquidity::Liquidity,
            percentage::Percentage,
            sqrt_price::{calculate_sqrt_price, SqrtPrice},
            token_amount::TokenAmount,
        },
        MIN_SQRT_PRICE,
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, init_basic_pool, init_dex_and_tokens, mint, quote, swap,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn max_tick_cross(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client, InvariantRef, TokenRef);
        init_basic_pool!(client, InvariantRef, TokenRef, dex, token_x, token_y);

        let mint_amount = u128::MAX;
        let alice = ink_e2e::alice();
        approve!(client, TokenRef, token_x, dex, mint_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, mint_amount, alice).unwrap();

        let liquidity = Liquidity::from_integer(10000000);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

        for i in (-2560..20).step_by(10) {
            let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

            let slippage_limit_lower = pool.sqrt_price;
            let slippage_limit_upper = pool.sqrt_price;

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                i,
                i + 10,
                liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
                alice
            )
            .unwrap();
        }

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();
        assert_eq!(pool.liquidity, liquidity);

        let amount = 760_000;
        let bob = ink_e2e::bob();
        mint!(client, TokenRef, token_x, address_of!(Bob), amount, alice).unwrap();
        let amount_x = balance_of!(client, TokenRef, token_x, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!(client, TokenRef, token_x, dex, amount, bob).unwrap();

        let pool_before = get_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            pool_key.fee_tier
        )
        .unwrap();

        let swap_amount = TokenAmount::new(amount);
        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        let quote_result = quote!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            slippage
        )
        .unwrap();

        let pool_after_quote = get_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            pool_key.fee_tier
        )
        .unwrap();

        let crosses_after_quote =
            ((pool_after_quote.current_tick_index - pool_before.current_tick_index) / 10).abs();
        assert_eq!(crosses_after_quote, 0);
        assert_eq!(quote_result.ticks.len() - 1, 145);

        swap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            slippage,
            bob
        )
        .unwrap();

        let pool_after = get_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            pool_key.fee_tier
        )
        .unwrap();

        let crosses = ((pool_after.current_tick_index - pool_before.current_tick_index) / 10).abs();
        assert_eq!(crosses, 146);
        assert_eq!(
            pool_after.current_tick_index,
            get_tick_at_sqrt_price(quote_result.target_sqrt_price, 10).unwrap()
        );

        Ok(())
    }
}
