#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{
            entrypoints::InvariantTrait, position_to_tick, FeeTier, PoolKey, LIQUIDITY_TICK_LIMIT,
        },
        invariant::InvariantRef,
        math::types::{
            liquidity::Liquidity,
            percentage::Percentage,
            sqrt_price::{calculate_sqrt_price, SqrtPrice},
        },
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens,
        get_liquidity_ticks, get_liquidity_ticks_amount, get_tick, get_tickmap,
        liquidity_tick_equals,
    };
    use token::Token;
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
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

        let ticks_amount = get_liquidity_ticks_amount!(client, dex, pool_key, -10, 10).unwrap();
        assert_eq!(ticks_amount, 2);

        let tickmap = get_tickmap!(client, dex, pool_key, -10, 10, false, alice);
        assert_eq!(tickmap.len(), 2);
        let mut ticks = vec![];
        tickmap.iter().for_each(|(chunk_index, chunk)| {
            for i in 0..64 {
                if chunk & (1 << i) != 0 {
                    ticks.push(position_to_tick(
                        *chunk_index,
                        i,
                        pool_key.fee_tier.tick_spacing,
                    ));
                }
            }
        });
        assert_eq!(ticks, vec![-10i32, 10]);

        let result = get_liquidity_ticks!(client, dex, pool_key, ticks.clone()).unwrap();
        assert_eq!(result.len(), 2);

        let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

        liquidity_tick_equals!(lower_tick, result[0]);
        liquidity_tick_equals!(upper_tick, result[1]);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_different_tick_spacings(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier_1 = FeeTier::new(Percentage::from_scale(1, 2), 2).unwrap();
        let fee_tier_2 = FeeTier::new(Percentage::from_scale(1, 2), 10).unwrap();

        add_fee_tier!(client, dex, fee_tier_1, alice).unwrap();
        add_fee_tier!(client, dex, fee_tier_2, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
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
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier_2,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let pool_key_1 = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier_1).unwrap();
        create_position!(
            client,
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

        let pool_key_2 = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier_2).unwrap();
        create_position!(
            client,
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

        let start_index_1 = -10;
        let end_index_1 = 30;

        let start_index_2 = -20;
        let end_index_2 = 40;
        let result =
            get_liquidity_ticks_amount!(client, dex, pool_key_1, start_index_1, end_index_1)
                .unwrap();
        assert_eq!(result, 2);
        let result = get_liquidity_ticks!(client, dex, pool_key_1, vec![-10, 30]).unwrap();
        assert_eq!(result.len(), 2);

        let result =
            get_liquidity_ticks_amount!(client, dex, pool_key_2, start_index_2, end_index_2)
                .unwrap();
        assert_eq!(result, 2);
        let result = get_liquidity_ticks!(client, dex, pool_key_2, vec![-20, 40]).unwrap();
        assert_eq!(result.len(), 2);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
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

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        let mut ticks = vec![];
        for i in 1..=LIQUIDITY_TICK_LIMIT / 2 {
            ticks.push(i as i32);
            ticks.push(-(i as i32));

            create_position!(
                client,
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

        let result = get_liquidity_ticks_amount!(
            client,
            dex,
            pool_key,
            -(LIQUIDITY_TICK_LIMIT as i32),
            LIQUIDITY_TICK_LIMIT as i32
        )
        .unwrap();
        assert_eq!(result, LIQUIDITY_TICK_LIMIT as u32);
        let result = get_liquidity_ticks!(client, dex, pool_key, ticks.clone()).unwrap();
        assert_eq!(result.len(), LIQUIDITY_TICK_LIMIT);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_limit_with_spread(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
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

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let spread = 64;
        let mut ticks = vec![];
        for i in 1..=LIQUIDITY_TICK_LIMIT / 2 {
            let index = (i * spread) as i32;
            ticks.push(index);
            ticks.push(-index);

            create_position!(
                client,
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

        let result = get_liquidity_ticks_amount!(
            client,
            dex,
            pool_key,
            -((LIQUIDITY_TICK_LIMIT * spread) as i32) / 2,
            (LIQUIDITY_TICK_LIMIT * spread) as i32 / 2
        )
        .unwrap();
        assert_eq!(result, LIQUIDITY_TICK_LIMIT as u32);
        let result = get_liquidity_ticks!(client, dex, pool_key, ticks.clone()).unwrap();
        assert_eq!(result.len(), LIQUIDITY_TICK_LIMIT);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_liquidity_ticks_with_offset(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(1, 2));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
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

        let result = get_liquidity_ticks_amount!(client, dex, pool_key, -10, 10).unwrap();
        assert_eq!(result, 2);

        let result_1 = get_liquidity_ticks!(client, dex, pool_key, vec![-10i32, 10]).unwrap();
        assert_eq!(result_1.len(), 2);

        let result_2 = get_liquidity_ticks!(client, dex, pool_key, vec![10]).unwrap();
        assert_eq!(result_2.len(), 1);

        assert_eq!(result_1[1], result_2[0]);

        Ok(())
    }
}
