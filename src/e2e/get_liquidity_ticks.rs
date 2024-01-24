#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey, LIQUIDITY_TICK_LIMIT},
        invariant::InvariantRef,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::types::{
        liquidity::Liquidity,
        percentage::Percentage,
        sqrt_price::{calculate_sqrt_price, SqrtPrice},
    };
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens,
        get_liquidity_ticks, get_liquidity_ticks_amount, get_tick, liquidity_tick_equals,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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

        let result = get_liquidity_ticks_amount!(client, InvariantRef, dex, pool_key);
        assert_eq!(result, 2);
        let result = get_liquidity_ticks!(client, InvariantRef, dex, pool_key, 0);
        assert_eq!(result.len(), 2);

        let lower_tick = get_tick!(client, InvariantRef, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, InvariantRef, dex, pool_key, 10).unwrap();

        liquidity_tick_equals!(lower_tick, result[0]);
        liquidity_tick_equals!(upper_tick, result[1]);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_different_tick_spacings(
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

        let result = get_liquidity_ticks_amount!(client, InvariantRef, dex, pool_key_1);
        assert_eq!(result, 2);
        let result = get_liquidity_ticks!(client, InvariantRef, dex, pool_key_1, 0);
        assert_eq!(result.len(), 2);

        let result = get_liquidity_ticks_amount!(client, InvariantRef, dex, pool_key_2);
        assert_eq!(result, 2);
        let result = get_liquidity_ticks!(client, InvariantRef, dex, pool_key_2, 0);
        assert_eq!(result.len(), 2);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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
        for i in 1..=LIQUIDITY_TICK_LIMIT / 2 {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                -(i as i32),
                i as i32,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let result = get_liquidity_ticks_amount!(client, InvariantRef, dex, pool_key);
        assert_eq!(result, LIQUIDITY_TICK_LIMIT as u32);
        let result = get_liquidity_ticks!(client, InvariantRef, dex, pool_key, 0);
        assert_eq!(result.len(), LIQUIDITY_TICK_LIMIT);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_limit_with_spread(
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
        let spread = 64;
        for i in 1..=LIQUIDITY_TICK_LIMIT / 2 {
            let index = (i * spread) as i32;

            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                -index,
                index,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let result = get_liquidity_ticks_amount!(client, InvariantRef, dex, pool_key);
        assert_eq!(result, LIQUIDITY_TICK_LIMIT as u32);
        let result = get_liquidity_ticks!(client, InvariantRef, dex, pool_key, 0);
        assert_eq!(result.len(), LIQUIDITY_TICK_LIMIT);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_with_offset(
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

        let result = get_liquidity_ticks_amount!(client, InvariantRef, dex, pool_key);
        assert_eq!(result, 2);

        let result_1 = get_liquidity_ticks!(client, InvariantRef, dex, pool_key, 0);
        assert_eq!(result_1.len(), 2);

        let result_2 = get_liquidity_ticks!(client, InvariantRef, dex, pool_key, 1);
        assert_eq!(result_2.len(), 1);

        assert_eq!(result_1[1], result_2[0]);

        Ok(())
    }
}
