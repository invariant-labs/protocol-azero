#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contract::ContractRef,
        contracts::{entrypoints::Invariant, FeeTier, PoolKey},
        math::types::{
            liquidity::Liquidity, percentage::Percentage, sqrt_price::sqrt_price::SqrtPrice,
        },
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens, get_pool,
        init_slippage_dex_and_tokens, init_slippage_pool_with_liquidity,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn position_slippage_zero_slippage_and_inside_range(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
        let pool_key = init_slippage_pool_with_liquidity!(
            client,
            ContractRef,
            TokenRef,
            dex,
            token_x,
            token_y
        );

        let pool = get_pool!(
            client,
            ContractRef,
            dex,
            token_x,
            token_y,
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
                ContractRef,
                dex,
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                known_price,
                known_price,
                alice
            );
        }
        // inside range
        {
            let liquidity_delta = Liquidity::from_integer(1_000_000);
            let limit_lower = SqrtPrice::new(994734637981406576896367);
            let limit_upper = SqrtPrice::new(1025038048074314166333500);

            let tick = pool_key.fee_tier.tick_spacing as i32;

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                limit_lower,
                limit_upper,
                alice
            );
        }

        Ok(())
    }

    #[ink_e2e::test]
    #[should_panic]
    async fn position_slippage_below_range(mut client: ink_e2e::Client<C, E>) -> () {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
        let pool_key = init_slippage_pool_with_liquidity!(
            client,
            ContractRef,
            TokenRef,
            dex,
            token_x,
            token_y
        );

        get_pool!(
            client,
            ContractRef,
            dex,
            token_x,
            token_y,
            pool_key.fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(1014432353584998786339859);
        let limit_upper = SqrtPrice::new(1045335831204498605270797);
        let tick = pool_key.fee_tier.tick_spacing as i32;
        create_position!(
            client,
            ContractRef,
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

    #[ink_e2e::test]
    #[should_panic]
    async fn position_slippage_above_range(mut client: ink_e2e::Client<C, E>) -> () {
        let alice = ink_e2e::alice();
        let (dex, token_x, token_y) = init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
        let pool_key = init_slippage_pool_with_liquidity!(
            client,
            ContractRef,
            TokenRef,
            dex,
            token_x,
            token_y
        );

        get_pool!(
            client,
            ContractRef,
            dex,
            token_x,
            token_y,
            pool_key.fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(955339206774222158009382);
        let limit_upper = SqrtPrice::new(984442481813945288458906);
        let tick = pool_key.fee_tier.tick_spacing as i32;
        create_position!(
            client,
            ContractRef,
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
}
