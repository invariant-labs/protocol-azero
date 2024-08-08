#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
        math::{
            log::get_tick_at_sqrt_price,
            types::{
                liquidity::Liquidity,
                percentage::Percentage,
                sqrt_price::{calculate_sqrt_price, SqrtPrice},
                token_amount::TokenAmount,
            },
            MIN_SQRT_PRICE,
        },
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, init_basic_pool, init_dex_and_tokens, mint, quote, swap,
    };
    use token::PSP22Mintable;
    use token::Token;
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn max_tick_cross(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);

        let mint_amount = u128::MAX;
        let alice = ink_e2e::alice();
        approve!(client, token_x, dex.account_id, mint_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, mint_amount, alice).unwrap();

        let liquidity = Liquidity::from_integer(10000000);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        for i in (-2560..20).step_by(10) {
            let pool = get_pool!(
                client,
                dex,
                token_x.account_id,
                token_y.account_id,
                fee_tier
            )
            .unwrap();

            let slippage_limit_lower = pool.sqrt_price;
            let slippage_limit_upper = pool.sqrt_price;

            create_position!(
                client,
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

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.liquidity, liquidity);

        let amount = 660_900;
        let bob = ink_e2e::bob();
        mint!(client, token_x, address_of!(Bob), amount, bob).unwrap();
        let amount_x = balance_of!(client, token_x, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!(client, token_x, dex.account_id, amount, bob).unwrap();

        let pool_before = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        let swap_amount = TokenAmount::new(amount);
        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        let quote_result =
            quote!(client, dex, pool_key, true, swap_amount, true, slippage).unwrap();

        let pool_after_quote = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        let crosses_after_quote =
            ((pool_after_quote.current_tick_index - pool_before.current_tick_index) / 10).abs();
        assert_eq!(crosses_after_quote, 0);
        assert_eq!(quote_result.ticks.len(), 128);

        swap!(
            client,
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
            dex,
            token_x.account_id,
            token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        let crosses = ((pool_after.current_tick_index - pool_before.current_tick_index) / 10).abs();
        assert_eq!(crosses, 128);
        assert_eq!(
            pool_after.current_tick_index,
            get_tick_at_sqrt_price(quote_result.target_sqrt_price, 10).unwrap()
        );

        Ok(())
    }
}
