#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::tickmap::get_max_chunk,
        contracts::{entrypoints::InvariantTrait, FeeTier, PoolKey},
        invariant::InvariantRef,
        math::types::liquidity::Liquidity,
        math::types::percentage::Percentage,
        math::types::sqrt_price::{calculate_sqrt_price, get_max_tick, get_min_tick, SqrtPrice},
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, approve, create_dex, create_pool, create_position, create_tokens, get_pool,
        get_tickmap,
    };
    use token::Token;
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
        let dex = create_dex!(client, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            dex,
            pool_key,
            -47,
            16,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let tickmap = get_tickmap!(
            client,
            dex,
            pool_key,
            get_min_tick(fee_tier.tick_spacing),
            get_max_tick(fee_tier.tick_spacing),
            false,
            alice
        );

        assert_eq!(
            tickmap[0],
            (
                10397,
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
        let dex = create_dex!(client, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
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
            dex,
            pool_key,
            get_min_tick(fee_tier.tick_spacing),
            get_max_tick(fee_tier.tick_spacing),
            false,
            alice
        );

        assert_eq!(tickmap[0], (0, 0b1));
        assert_eq!(
            tickmap[1],
            (1039, 0b1100000000000000000000000000000000000000000000000000)
        );

        assert_eq!(
            tickmap[2],
            (
                get_max_chunk(fee_tier.tick_spacing),
                0b10000000000000000000000000000000000
            )
        );
        assert_eq!(tickmap.len(), 3);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_edge_ticks_initialized(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::new(1000);

        create_position!(
            client,
            dex,
            pool_key,
            get_min_tick(fee_tier.tick_spacing),
            get_min_tick(fee_tier.tick_spacing) + 1,
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            get_max_tick(fee_tier.tick_spacing) - 1,
            get_max_tick(fee_tier.tick_spacing),
            liquidity_delta,
            pool.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();
        {
            let tickmap = get_tickmap!(
                client,
                dex,
                pool_key,
                get_min_tick(fee_tier.tick_spacing),
                get_max_tick(fee_tier.tick_spacing),
                false,
                alice
            );

            assert_eq!(tickmap[0], (0, 0b11));
            assert_eq!(
                tickmap[1],
                (
                    get_max_chunk(fee_tier.tick_spacing),
                    0b1100000000000000000000000000000
                )
            );
            assert_eq!(tickmap.len(), 2);

            let tickmap = get_tickmap!(
                client,
                dex,
                pool_key,
                get_min_tick(fee_tier.tick_spacing),
                get_max_tick(fee_tier.tick_spacing),
                true,
                alice
            );
            assert_eq!(
                tickmap[0],
                (
                    get_max_chunk(fee_tier.tick_spacing),
                    0b1100000000000000000000000000000
                )
            );
            assert_eq!(tickmap[1], (0, 0b11));
            assert_eq!(tickmap.len(), 2);
        }

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_more_chunks_above(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::new(1000);

        for i in (6..52500).step_by(64) {
            create_position!(
                client,
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
            dex,
            pool_key,
            get_min_tick(fee_tier.tick_spacing),
            get_max_tick(fee_tier.tick_spacing),
            false,
            alice
        );

        for (i, _) in (0..tickmap.len()).enumerate() {
            let current = 10397 + i as u16;
            assert_eq!(
                tickmap[i],
                (
                    current,
                    0b1100000000000000000000000000000000000000000000000000000
                )
            );
        }
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_more_chunks_below(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::new(1000);

        for i in (-52544..6).step_by(64) {
            create_position!(
                client,
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
            dex,
            pool_key,
            get_min_tick(fee_tier.tick_spacing),
            get_max_tick(fee_tier.tick_spacing),
            false,
            alice
        );
        for (i, _) in (0..tickmap.len()).enumerate() {
            let current = 9576 + i as u16;
            assert_eq!(
                tickmap[i],
                (current, 0b1100000000000000000000000000000000000000000000000)
            );
        }
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_get_tickmap_max_chunks_returned(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let dex = create_dex!(client, Percentage::new(0));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        approve!(client, token_x, dex.account_id, initial_amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_amount, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let init_tick = -200_000;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert!(result.is_ok());

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = Liquidity::new(1000);

        for i in (0..104832).step_by(64) {
            create_position!(
                client,
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
            dex,
            pool_key,
            get_min_tick(fee_tier.tick_spacing),
            get_max_tick(fee_tier.tick_spacing),
            false,
            alice
        );

        assert_eq!(tickmap.len(), 1638);

        Ok(())
    }
}
