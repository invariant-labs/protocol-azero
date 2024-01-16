#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey, Tick},
        invariant::InvariantRef,
        math::{
            types::{
                liquidity::Liquidity,
                percentage::Percentage,
                sqrt_price::{calculate_sqrt_price, SqrtPrice},
            },
            MAX_TICK,
        },
    };
    use decimal::*;
    use ink_e2e::build_message;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens,
        get_all_ticks, get_tick,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_get_all_ticks(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
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

        approve!(client, TokenRef, token_x, dex, 500, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        create_position!(
            client,
            InvariantRef,
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

        let expected_ticks = vec![
            get_tick!(client, InvariantRef, dex, pool_key, -10).unwrap(),
            get_tick!(client, InvariantRef, dex, pool_key, 10).unwrap(),
        ];
        let result = get_all_ticks!(client, InvariantRef, dex, pool_key, -MAX_TICK, 2);
        assert_eq!(result.0, expected_ticks);
        assert!(!result.1);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_all_ticks_different_tick_spacings(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier_1 = FeeTier::new(Percentage::from_scale(1, 2), 2).unwrap();
        let fee_tier_2 = FeeTier::new(Percentage::from_scale(1, 2), 10).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier_1, alice).unwrap();
        add_fee_tier!(client, InvariantRef, dex, fee_tier_2, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier_1,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier_2,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let pool_key_1 = PoolKey::new(token_x, token_y, fee_tier_1).unwrap();
        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key_1,
            -10,
            30,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let pool_key_2 = PoolKey::new(token_x, token_y, fee_tier_2).unwrap();
        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key_2,
            -20,
            40,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let expected_ticks = vec![
            get_tick!(client, InvariantRef, dex, pool_key_1, -10).unwrap(),
            get_tick!(client, InvariantRef, dex, pool_key_1, 30).unwrap(),
        ];
        let result = get_all_ticks!(client, InvariantRef, dex, pool_key_1, -MAX_TICK, 2);
        assert_eq!(result.0, expected_ticks);
        assert!(!result.1);

        let expected_ticks = vec![
            get_tick!(client, InvariantRef, dex, pool_key_2, -20).unwrap(),
            get_tick!(client, InvariantRef, dex, pool_key_2, 40).unwrap(),
        ];
        let result = get_all_ticks!(client, InvariantRef, dex, pool_key_2, -MAX_TICK, 2);
        assert_eq!(result.0, expected_ticks);
        assert!(!result.1);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_all_ticks_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
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

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        for i in 1..=88 {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                -i,
                i,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let result = get_all_ticks!(client, InvariantRef, dex, pool_key, -MAX_TICK, 176);
        assert_eq!(result.0.len(), 176);
        assert!(!result.1);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_all_ticks_limit_with_spread(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
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

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let max_tick_spread = 2520; // 221818 / 88
        for i in 1..=88 {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                -i * max_tick_spread,
                i * max_tick_spread,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let result = get_all_ticks!(client, InvariantRef, dex, pool_key, -MAX_TICK, 176);
        assert_eq!(result.0.len(), 176);
        assert!(!result.1);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_all_ticks_with_offset(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
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

        approve!(client, TokenRef, token_x, dex, 500, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, 500, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        create_position!(
            client,
            InvariantRef,
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

        let expected_ticks = vec![get_tick!(client, InvariantRef, dex, pool_key, 10).unwrap()];
        let result = get_all_ticks!(client, InvariantRef, dex, pool_key, -9, 1);
        assert_eq!(result.0, expected_ticks);
        assert!(!result.1);

        let result = get_all_ticks!(client, InvariantRef, dex, pool_key, -9, 2);
        assert_eq!(result.0, expected_ticks);
        assert!(result.1);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_all_ticks_multiple_queries(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, InvariantRef, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
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

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        for i in 1..=500 {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                -i,
                i,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let mut end = false;
        let mut ticks: Vec<Tick> = vec![];

        while !end {
            let index = if let Some(last_tick) = ticks.last() {
                last_tick.index + 1
            } else {
                -MAX_TICK
            };

            let mut result = get_all_ticks!(client, InvariantRef, dex, pool_key, index, 100);
            ticks.append(&mut result.0);
            end = result.1;
        }

        assert_eq!(ticks.len(), 1000);

        Ok(())
    }
}
