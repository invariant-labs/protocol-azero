#[cfg(test)]
pub mod e2e_tests {
    use crate::{
        contracts::tickmap::get_max_chunk,
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
    };
    use decimal::*;
    use ink_e2e::build_message;
    use math::consts::{MAX_TICK, MIN_TICK};
    use math::types::liquidity::Liquidity;
    use math::types::percentage::Percentage;
    use math::types::sqrt_price::{calculate_sqrt_price, get_max_tick, get_min_tick, SqrtPrice};
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens, get_pool,
        get_tickmap,
    };
    use token::TokenRef;
    use token::PSP22;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    fn _to_binary(v: (u16, u64)) {
        println!(
            "Chunk Index = {:?} Value = {:?}, Binary = {:b}",
            v.0, v.1, v.1
        );
    }

    #[ink_e2e::test]
    async fn test_get_tickmap(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            -58,
            5,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        assert_eq!(
            tickmap[0],
            (
                3465,
                0b1000000000000000000000000000000000000000000000000000000000000001
            )
        );
        assert_eq!(tickmap.len(), 1);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_tick_spacing_over_one(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            10,
            20,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            get_min_tick(fee_tier.tick_spacing),
            get_max_tick(fee_tier.tick_spacing),
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        assert_eq!(tickmap[0], (0, 0b1));
        assert_eq!(
            tickmap[1],
            (346, 0b1100000000000000000000000000000000000000)
        );
        assert_eq!(
            tickmap[2],
            (get_max_chunk(fee_tier.tick_spacing), 0b10000000000)
        );
        assert_eq!(tickmap.len(), 3);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_edge_ticks_intialized(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            -221818,
            -221817,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            InvariantRef,
            dex,
            pool_key,
            221817,
            221818,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        assert_eq!(tickmap[0], (0, 0b11));
        assert_eq!(
            tickmap[1],
            (
                get_max_chunk(fee_tier.tick_spacing),
                0b11000000000000000000000000000000000000000000000000000
            )
        );
        assert_eq!(tickmap.len(), 2);
        {
            let tickmap = get_tickmap!(client, InvariantRef, dex, pool_key, MIN_TICK, alice);
            assert_eq!(tickmap[0], (0, 0b11));
            assert_eq!(
                tickmap[1],
                (
                    get_max_chunk(fee_tier.tick_spacing),
                    0b11000000000000000000000000000000000000000000000000000
                )
            );
            assert_eq!(tickmap.len(), 2);

            let tickmap = get_tickmap!(client, InvariantRef, dex, pool_key, 0, alice);
            assert_eq!(tickmap[0], (0, 0b11));
            assert_eq!(
                tickmap[1],
                (
                    get_max_chunk(fee_tier.tick_spacing),
                    0b11000000000000000000000000000000000000000000000000000
                )
            );
            assert_eq!(tickmap.len(), 2);

            let tickmap = get_tickmap!(client, InvariantRef, dex, pool_key, MAX_TICK, alice);
            assert_eq!(tickmap[0], (0, 0b11));
            assert_eq!(
                tickmap[1],
                (
                    get_max_chunk(fee_tier.tick_spacing),
                    0b11000000000000000000000000000000000000000000000000000
                )
            );
            assert_eq!(tickmap.len(), 2);
        }

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_more_chunks_above(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        for i in (6..52500).step_by(64) {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                i,
                i + 1,
                liquidity_delta,
                pool.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        for (i, _) in (0..tickmap.len()).enumerate() {
            let current = 3466 + i as u16;
            assert_eq!(tickmap[i], (current, 0b11));
        }
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_more_chunks_below(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        for i in (-52544..6).step_by(64) {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                i,
                i + 1,
                liquidity_delta,
                pool.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );
        for (i, _) in (0..tickmap.len()).enumerate() {
            let current = 2644 + i as u16;
            assert_eq!(
                tickmap[i],
                (
                    current,
                    0b110000000000000000000000000000000000000000000000000000000000
                )
            );
        }
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_max_chunks_returned(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, InvariantRef, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, TokenRef, initial_amount, initial_amount);

        approve!(client, TokenRef, token_x, dex, initial_amount, alice).unwrap();
        approve!(client, TokenRef, token_y, dex, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let init_tick = -200_000;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, InvariantRef, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            InvariantRef,
            dex,
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(client, InvariantRef, dex, token_x, token_y, fee_tier).unwrap();

        let liquidity_delta = Liquidity::new(1000);

        for i in (0..104832).step_by(64) {
            create_position!(
                client,
                InvariantRef,
                dex,
                pool_key,
                i,
                i + 1,
                liquidity_delta,
                pool.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
        }

        let tickmap = get_tickmap!(
            client,
            InvariantRef,
            dex,
            pool_key,
            pool.current_tick_index,
            alice
        );

        assert_eq!(tickmap.len(), 1638);

        Ok(())
    }
}
