#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, InvariantError},
        invariant::InvariantRef,
        math::types::percentage::Percentage,
        math::types::sqrt_price::{calculate_sqrt_price, SqrtPrice},
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{add_fee_tier, create_dex, create_pool, create_tokens, get_pool};
    use token::TokenRef;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_create_pool(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let alice = ink_e2e::alice();

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

        get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_x_to_y_and_y_to_x(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let alice = ink_e2e::alice();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

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

        get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let result = create_pool!(
            client,
            dex,
            token_y.account_id,
            token_x.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );
        assert_eq!(result, Err(InvariantError::PoolAlreadyExist));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_with_same_tokens(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, _) = create_tokens!(client, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let alice = ink_e2e::alice();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let result = create_pool!(
            client,
            dex,
            token_x.account_id,
            token_x.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        );

        assert_eq!(result, Err(InvariantError::TokensAreSame));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_fee_tier_not_added(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let alice = ink_e2e::alice();

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

        assert_eq!(result, Err(InvariantError::FeeTierNotFound));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_init_tick_not_divided_by_tick_spacing(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);
        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();
        let init_tick = 2;
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

        assert_eq!(result, Err(InvariantError::InvalidInitTick));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_init_sqrt_price_minimal_difference_from_tick(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);
        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap() + SqrtPrice::new(1);
        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

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

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.current_tick_index, init_tick);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_init_sqrt_price_has_closer_init_tick(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);
        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();
        let init_tick = 2;
        // tick = 3 -> 1.000150003749000000000000
        // between  -> 1.000175003749000000000000
        // tick = 4 -> 1.000200010000000000000000
        let init_sqrt_price = SqrtPrice::new(1000175003749000000000000);
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
        assert_eq!(result, Err(InvariantError::InvalidInitSqrtPrice));
        let correct_init_tick = 3;
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            correct_init_tick,
            alice
        )
        .unwrap();

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.current_tick_index, correct_init_tick);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_pool_init_sqrt_price_has_closer_init_tick_spacing_over_one(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);
        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();
        let init_tick = 0;
        // tick = 3 -> 1.000150003749000000000000
        // between  -> 1.000225003749000000000000
        // tick = 6 -> 1.000300030001000000000000
        let init_sqrt_price = SqrtPrice::new(1000225003749000000000000);
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
        assert_eq!(result, Err(InvariantError::InvalidInitSqrtPrice));

        let correct_init_tick = 3;
        create_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            correct_init_tick,
            alice
        )
        .unwrap();

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.current_tick_index, correct_init_tick);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_create_many_pools_success(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));

        let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let alice = ink_e2e::alice();

        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        // size of PoolKeys = 32+32+8+2 = 74 bytes, max size of ink! storage cell is 16384 bytes
        // that is 16384 - 32 (first 32B is the size of vec) = 16352 bytes left for memory
        // 16352 / 74 = 220 elements. Adding 221st element will panic because of not enough memory in the storage cell

        let amount_of_pools_to_create = 1000;

        for i in 0..amount_of_pools_to_create {
            let (token_x, token_y) = create_tokens!(client, 500, 500);
            println!("{i}");
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
            get_pool!(
                client,
                dex,
                token_x.account_id,
                token_y.account_id,
                fee_tier
            )
            .unwrap();
        }

        Ok(())
    }
}
