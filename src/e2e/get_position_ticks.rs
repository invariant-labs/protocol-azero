#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey, POSITION_TICK_LIMIT},
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
        add_fee_tier, address_of, approve, create_dex, create_pool, create_position, create_tokens,
        get_position_ticks, get_tick, position_tick_equals,
    };
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_get_position_ticks(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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

        let result = get_position_ticks!(client, InvariantRef, dex, address_of!(Alice), 0);
        assert_eq!(result.len(), 2);

        let lower_tick = get_tick!(client, InvariantRef, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, InvariantRef, dex, pool_key, 10).unwrap();

        position_tick_equals!(result[0], lower_tick);
        position_tick_equals!(result[1], upper_tick);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_position_ticks_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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
        for i in 1..=POSITION_TICK_LIMIT / 2 {
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

        let result = get_position_ticks!(client, InvariantRef, dex, address_of!(Alice), 0);
        assert_eq!(result.len(), POSITION_TICK_LIMIT);

        for i in 1..=POSITION_TICK_LIMIT / 2 {
            let lower_tick = get_tick!(client, InvariantRef, dex, pool_key, -(i as i32)).unwrap();
            let upper_tick = get_tick!(client, InvariantRef, dex, pool_key, i as i32).unwrap();

            position_tick_equals!(result[i * 2 - 2], lower_tick);
            position_tick_equals!(result[i * 2 - 1], upper_tick);
        }

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_position_ticks_with_offset(
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

        let result_1 = get_position_ticks!(client, InvariantRef, dex, address_of!(Alice), 0);
        assert_eq!(result_1.len(), 4);

        let result_2 = get_position_ticks!(client, InvariantRef, dex, address_of!(Alice), 1);
        assert_eq!(result_2.len(), 2);

        assert_eq!(result_1[2], result_2[0]);
        assert_eq!(result_1[3], result_2[1]);

        Ok(())
    }
}
