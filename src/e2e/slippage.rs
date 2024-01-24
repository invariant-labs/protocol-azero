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
            liquidity::Liquidity,
            percentage::Percentage,
            sqrt_price::{calculate_sqrt_price, SqrtPrice},
            token_amount::TokenAmount,
        },
        MAX_SQRT_PRICE, MIN_SQRT_PRICE,
    };
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, init_basic_pool, init_basic_position, init_dex_and_tokens,
        init_slippage_dex_and_tokens, init_slippage_pool_with_liquidity, mint, quote, swap,
        swap_exact_limit,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_basic_slippage(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client, InvariantRef, TokenRef);
        let pool_key = init_slippage_pool_with_liquidity!(
            client,
            InvariantRef,
            TokenRef,
            dex,
            token_x,
            token_y
        );
        let amount = 10u128.pow(8);
        let swap_amount = TokenAmount::new(amount);
        approve!(client, TokenRef, token_x, dex, amount, alice).unwrap();

        let target_sqrt_price = SqrtPrice::new(1009940000000000000000001);
        swap!(
            client,
            InvariantRef,
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
            InvariantRef,
            dex,
            token_x,
            token_y,
            pool_key.fee_tier
        )
        .unwrap();

        assert_eq!(expected_sqrt_price, pool.sqrt_price);
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_swap_close_to_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client, InvariantRef, TokenRef);
        let pool_key = init_slippage_pool_with_liquidity!(
            client,
            InvariantRef,
            TokenRef,
            dex,
            token_x,
            token_y
        );
        let amount = 10u128.pow(8);
        let swap_amount = TokenAmount::new(amount);
        approve!(client, TokenRef, token_x, dex, amount, alice).unwrap();

        let target_sqrt_price = SqrtPrice::new(MAX_SQRT_PRICE);
        let quoted_target_sqrt_price = quote!(
            client,
            InvariantRef,
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
            InvariantRef,
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
        let (dex, token_x, token_y) = init_dex_and_tokens!(client, InvariantRef, TokenRef);
        init_basic_pool!(client, InvariantRef, TokenRef, dex, token_x, token_y);
        init_basic_position!(client, InvariantRef, TokenRef, dex, token_x, token_y);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

        let amount = 1000;
        let bob = ink_e2e::bob();
        let alice = ink_e2e::alice();
        mint!(client, TokenRef, token_x, address_of!(Bob), amount, alice).unwrap();
        let amount_x = balance_of!(client, TokenRef, token_x, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!(client, TokenRef, token_x, dex, amount, bob).unwrap();

        let swap_amount = TokenAmount::new(amount);
        swap_exact_limit!(
            client,
            InvariantRef,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            bob
        );

        Ok(())
    }
}
