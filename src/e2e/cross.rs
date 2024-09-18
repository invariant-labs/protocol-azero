#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
        math::{
            types::{
                fee_growth::FeeGrowth,
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
        create_tokens, get_pool, get_tick, init_basic_pool, init_basic_position,
        init_cross_position, init_cross_swap, init_dex_and_tokens, mint, swap,
    };
    use token::PSP22Mintable;
    use token::Token;
    use token::TokenRef;
    use token::PSP22;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_cross(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);
        init_cross_position!(client, dex, token_x, token_y);
        init_cross_swap!(client, dex, token_x, token_y);

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        let upper_tick_index = 10;
        let middle_tick_index = -10;
        let lower_tick_index = -20;

        let upper_tick = get_tick!(client, dex, pool_key, upper_tick_index).unwrap();
        let middle_tick = get_tick!(client, dex, pool_key, middle_tick_index).unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, lower_tick_index).unwrap();

        assert_eq!(
            upper_tick.liquidity_change,
            Liquidity::from_integer(1000000)
        );
        assert_eq!(
            middle_tick.liquidity_change,
            Liquidity::from_integer(1000000)
        );
        assert_eq!(
            lower_tick.liquidity_change,
            Liquidity::from_integer(1000000)
        );

        assert_eq!(upper_tick.fee_growth_outside_x, FeeGrowth::new(0.into()));
        assert_eq!(
            middle_tick.fee_growth_outside_x,
            FeeGrowth::new(30000000000000000000000_u128.into())
        );
        assert_eq!(lower_tick.fee_growth_outside_x, FeeGrowth::new(0.into()));

        Ok(())
    }
}
