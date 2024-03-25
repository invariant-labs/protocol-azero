#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
        math::types::{
            liquidity::Liquidity,
            percentage::Percentage,
            sqrt_price::{calculate_sqrt_price, SqrtPrice},
        },
        InvariantError,
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens, get_pool,
        init_slippage_dex_and_tokens, init_slippage_pool_with_liquidity,
    };
    use token::Token;
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_position_slippage_zero_slippage_and_inside_range(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client);
        let pool_key = init_slippage_pool_with_liquidity!(client, dex, token_x, token_y);

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        // zero slippage
        {
            let liquidity_delta = Liquidity::from_integer(1_000_000);
            let known_price = pool.sqrt_price;
            let tick = pool_key.fee_tier.tick_spacing as i32;
            create_position!(
                client,
                dex,
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                known_price,
                known_price,
                alice
            )
            .unwrap();
        }
        // inside range
        {
            let liquidity_delta = Liquidity::from_integer(1_000_000);
            let limit_lower = SqrtPrice::new(994734637981406576896367);
            let limit_upper = SqrtPrice::new(1025038048074314166333500);

            let tick = pool_key.fee_tier.tick_spacing as i32;

            create_position!(
                client,
                dex,
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                limit_lower,
                limit_upper,
                alice
            )
            .unwrap();
        }

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_position_slippage_below_range(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client);
        let pool_key = init_slippage_pool_with_liquidity!(client, dex, token_x, token_y);

        get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(1014432353584998786339859);
        let limit_upper = SqrtPrice::new(1045335831204498605270797);
        let tick = pool_key.fee_tier.tick_spacing as i32;
        let result = create_position!(
            client,
            dex,
            pool_key,
            -tick,
            tick,
            liquidity_delta,
            limit_lower,
            limit_upper,
            alice
        );

        assert_eq!(result, Err(InvariantError::PriceLimitReached));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_position_slippage_above_range(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client);
        let pool_key = init_slippage_pool_with_liquidity!(client, dex, token_x, token_y);

        get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(955339206774222158009382);
        let limit_upper = SqrtPrice::new(984442481813945288458906);
        let tick = pool_key.fee_tier.tick_spacing as i32;
        let result = create_position!(
            client,
            dex,
            pool_key,
            -tick,
            tick,
            liquidity_delta,
            limit_lower,
            limit_upper,
            alice
        );

        assert_eq!(result, Err(InvariantError::PriceLimitReached));

        Ok(())
    }
}
