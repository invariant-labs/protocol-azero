#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, InvariantError, PoolKey},
        invariant::InvariantRef,
        math::{
            types::{
                liquidity::Liquidity,
                percentage::Percentage,
                sqrt_price::{calculate_sqrt_price, SqrtPrice},
                token_amount::TokenAmount,
            },
            MAX_SQRT_PRICE, MIN_SQRT_PRICE,
        },
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, init_basic_pool, init_basic_position, init_dex_and_tokens,
        init_slippage_dex_and_tokens, init_slippage_pool_with_liquidity, mint, quote, swap,
        swap_exact_limit,
    };
    use token::Token;
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_basic_slippage(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client);
        let pool_key = init_slippage_pool_with_liquidity!(client, dex, token_x, token_y);
        let amount = 10u128.pow(8);
        let swap_amount = TokenAmount::new(amount);
        approve!(client, token_x, dex.account_id, amount, alice).unwrap();

        let target_sqrt_price = SqrtPrice::new(1009940000000000000000001);
        swap!(
            client,
            dex,
            pool_key,
            false,
            swap_amount,
            true,
            target_sqrt_price,
            alice
        )
        .unwrap();
        let expected_sqrt_price = SqrtPrice::new(1009940000000000000000000);
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        assert_eq!(expected_sqrt_price, pool.sqrt_price);
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_swap_close_to_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client);
        let pool_key = init_slippage_pool_with_liquidity!(client, dex, token_x, token_y);
        let amount = 10u128.pow(8);
        let swap_amount = TokenAmount::new(amount);
        approve!(client, token_x, dex.account_id, amount, alice).unwrap();

        let target_sqrt_price = SqrtPrice::new(MAX_SQRT_PRICE);
        let quoted_target_sqrt_price = quote!(
            client,
            dex,
            pool_key,
            false,
            swap_amount,
            true,
            target_sqrt_price
        )
        .unwrap()
        .target_sqrt_price;

        let target_sqrt_price = quoted_target_sqrt_price - SqrtPrice::new(1);

        let result = swap!(
            client,
            dex,
            pool_key,
            false,
            swap_amount,
            true,
            target_sqrt_price,
            alice
        );
        assert_eq!(result, Err(InvariantError::PriceLimitReached));
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_swap_exact_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        let amount = 1000;
        let bob = ink_e2e::bob();
        let alice = ink_e2e::alice();
        mint!(client, token_x, address_of!(Bob), amount, alice).unwrap();
        let amount_x = balance_of!(client, token_x, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!(client, token_x, dex.account_id, amount, bob).unwrap();

        let swap_amount = TokenAmount::new(amount);
        swap_exact_limit!(client, dex, pool_key, true, swap_amount, true, bob);

        Ok(())
    }
}
