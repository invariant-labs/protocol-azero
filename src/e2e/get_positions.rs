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
    };
    use decimal::*;
    use ink::primitives::AccountId;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens,
        get_positions,
    };
    use token::Token;
    use token::TokenRef;
    use token::PSP22;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_get_positions(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 10;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, token_x, dex.account_id, 500, alice).unwrap();
        approve!(client, token_y, dex.account_id, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -10,
            10,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -20,
            20,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let result = get_positions!(client, dex, 2, 0, alice).unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.1, 2);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_positions_less_than_exist(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 10;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, token_x, dex.account_id, 500, alice).unwrap();
        approve!(client, token_y, dex.account_id, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -10,
            10,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -20,
            20,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let result = get_positions!(client, dex, 1, 0, alice).unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.1, 2);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_positions_more_than_exist(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 10;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, token_x, dex.account_id, 500, alice).unwrap();
        approve!(client, token_y, dex.account_id, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -10,
            10,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -20,
            20,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let result = get_positions!(client, dex, 3, 0, alice).unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.1, 2);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_positions_with_offset(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 10;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, token_x, dex.account_id, 500, alice).unwrap();
        approve!(client, token_y, dex.account_id, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -10,
            10,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -20,
            20,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let result = get_positions!(client, dex, 1, 1, alice).unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.1, 2);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_positions_with_offset_less_than_exist(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 10;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, token_x, dex.account_id, 500, alice).unwrap();
        approve!(client, token_y, dex.account_id, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -10,
            10,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -20,
            20,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -30,
            30,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let result = get_positions!(client, dex, 1, 1, alice).unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.1, 3);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_positions_with_offset_more_than_exist(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 10;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, token_x, dex.account_id, 500, alice).unwrap();
        approve!(client, token_y, dex.account_id, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -10,
            10,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -20,
            20,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let result = get_positions!(client, dex, 2, 1, alice).unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.1, 2);

        Ok(())
    }
}
