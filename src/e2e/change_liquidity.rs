#[cfg(test)]
pub mod e2e_tests {
    use crate::contracts::InvariantError;
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantEntrypoints, FeeTier, PoolKey},
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
        add_fee_tier, address_of, approve, balance_of, change_liquidity, create_dex, create_pool,
        create_position, create_tokens, get_pool, get_position, get_tick, remove_position,
    };
    use token::Token;
    use token::TokenRef;
    use token::PSP22;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_change_liquidity(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

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
            Liquidity::from_integer(10000),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let position = get_position!(client, dex, 0, alice).unwrap();
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

        assert_eq!(position.liquidity, Liquidity::from_integer(10000));
        assert_eq!(pool.liquidity, Liquidity::from_integer(10000));
        assert_eq!(lower_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(lower_tick.sign);

        assert_eq!(upper_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(!upper_tick.sign);
        // increase liquidity
        {
            let dex_balance_x_before = balance_of!(client, token_x, dex.account_id);
            let dex_balance_y_before = balance_of!(client, token_y, dex.account_id);
            assert_eq!(dex_balance_x_before, 5);
            assert_eq!(dex_balance_y_before, 5);

            let user_balance_x_before = balance_of!(client, token_x, address_of!(Alice));
            let user_balance_y_before = balance_of!(client, token_y, address_of!(Alice));

            change_liquidity!(
                client,
                dex,
                0,
                Liquidity::from_integer(10000),
                true,
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            let dex_balance_x = balance_of!(client, token_x, dex.account_id);
            let dex_balance_y = balance_of!(client, token_y, dex.account_id);
            assert_eq!(dex_balance_x, 10);
            assert_eq!(dex_balance_y, 10);
            let user_balance_x = balance_of!(client, token_x, address_of!(Alice));
            let user_balance_y = balance_of!(client, token_y, address_of!(Alice));

            assert_eq!(user_balance_x_before - user_balance_x, 5);
            assert_eq!(user_balance_y_before - user_balance_y, 5);

            let position = get_position!(client, dex, 0, alice).unwrap();
            let pool = get_pool!(
                client,
                dex,
                token_x.account_id,
                token_y.account_id,
                fee_tier
            )
            .unwrap();
            let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
            let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

            assert_eq!(position.liquidity, Liquidity::from_integer(20000));
            assert_eq!(pool.liquidity, Liquidity::from_integer(20000));
            assert_eq!(lower_tick.liquidity_change, Liquidity::from_integer(20000));
            assert!(lower_tick.sign);

            assert_eq!(upper_tick.liquidity_change, Liquidity::from_integer(20000));
            assert!(!upper_tick.sign);
        }
        // decrease liquidity
        {
            let dex_balance_x_before = balance_of!(client, token_x, dex.account_id);
            let dex_balance_y_before = balance_of!(client, token_y, dex.account_id);
            assert_eq!(dex_balance_x_before, 10);
            assert_eq!(dex_balance_y_before, 10);

            let user_balance_x_before = balance_of!(client, token_x, address_of!(Alice));
            let user_balance_y_before = balance_of!(client, token_y, address_of!(Alice));

            change_liquidity!(
                client,
                dex,
                0,
                Liquidity::from_integer(10000),
                false,
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();
            let dex_balance_x = balance_of!(client, token_x, dex.account_id);
            let dex_balance_y = balance_of!(client, token_y, dex.account_id);
            let user_balance_x = balance_of!(client, token_x, address_of!(Alice));
            let user_balance_y = balance_of!(client, token_y, address_of!(Alice));

            assert_eq!(dex_balance_x, 6);
            assert_eq!(dex_balance_y, 6);
            assert_eq!(user_balance_x - user_balance_x_before, 4);
            assert_eq!(user_balance_y - user_balance_y_before, 4);

            let position = get_position!(client, dex, 0, alice).unwrap();
            let pool = get_pool!(
                client,
                dex,
                token_x.account_id,
                token_y.account_id,
                fee_tier
            )
            .unwrap();
            let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
            let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

            assert_eq!(position.liquidity, Liquidity::from_integer(10000));
            assert_eq!(pool.liquidity, Liquidity::from_integer(10000));
            assert_eq!(lower_tick.liquidity_change, Liquidity::from_integer(10000));
            assert!(lower_tick.sign);

            assert_eq!(upper_tick.liquidity_change, Liquidity::from_integer(10000));
            assert!(!upper_tick.sign);
        }

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_liquidity_amount_is_zero(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

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
            Liquidity::from_integer(10000),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let position = get_position!(client, dex, 0, alice).unwrap();
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

        assert_eq!(position.liquidity, Liquidity::from_integer(10000));
        assert_eq!(pool.liquidity, Liquidity::from_integer(10000));
        assert_eq!(lower_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(lower_tick.sign);

        assert_eq!(upper_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(!upper_tick.sign);

        let result = change_liquidity!(
            client,
            dex,
            0,
            Liquidity::new(1),
            false,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        );
        assert_eq!(result, Err(InvariantError::AmountIsZero));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_liquidity_and_remove_position(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

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
            Liquidity::from_integer(10000),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let position = get_position!(client, dex, 0, alice).unwrap();
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

        assert_eq!(position.liquidity, Liquidity::from_integer(10000));
        assert_eq!(pool.liquidity, Liquidity::from_integer(10000));
        assert_eq!(lower_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(lower_tick.sign);

        assert_eq!(upper_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(!upper_tick.sign);

        change_liquidity!(
            client,
            dex,
            0,
            position.liquidity - Liquidity::new(1),
            false,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        remove_position!(client, dex, 0, alice).unwrap();

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_liquidity_zero_liquidity(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

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
            Liquidity::from_integer(10000),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let position = get_position!(client, dex, 0, alice).unwrap();
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

        assert_eq!(position.liquidity, Liquidity::from_integer(10000));
        assert_eq!(pool.liquidity, Liquidity::from_integer(10000));
        assert_eq!(lower_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(lower_tick.sign);

        assert_eq!(upper_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(!upper_tick.sign);

        let result = change_liquidity!(
            client,
            dex,
            0,
            position.liquidity,
            false,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        );
        assert_eq!(result, Err(InvariantError::ZeroLiquidity));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_liquidity_zero_liquidity_change(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

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
            Liquidity::from_integer(10000),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let position = get_position!(client, dex, 0, alice).unwrap();
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, -10).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, 10).unwrap();

        assert_eq!(position.liquidity, Liquidity::from_integer(10000));
        assert_eq!(pool.liquidity, Liquidity::from_integer(10000));
        assert_eq!(lower_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(lower_tick.sign);

        assert_eq!(upper_tick.liquidity_change, Liquidity::from_integer(10000));
        assert!(!upper_tick.sign);

        let result = change_liquidity!(
            client,
            dex,
            0,
            Liquidity::from_integer(0),
            true,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        );
        assert_eq!(result, Err(InvariantError::LiquidityChangeZero));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_liquidity_insufficient_balance(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));
        let (token_x, token_y) = create_tokens!(client, 500, 500);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

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

        approve!(client, token_x, dex.account_id, u128::MAX, alice).unwrap();
        approve!(client, token_y, dex.account_id, u128::MAX, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            -10,
            10,
            Liquidity::from_integer(10000),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        let result = change_liquidity!(
            client,
            dex,
            0,
            Liquidity::from_integer(1000000000),
            true,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        );
        assert_eq!(result, Err(InvariantError::TransferError));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_change_liquidity_no_position(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::new(0));

        let alice = ink_e2e::alice();

        let result = change_liquidity!(
            client,
            dex,
            0,
            Liquidity::from_integer(10000),
            true,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        );
        assert_eq!(result, Err(InvariantError::PositionNotFound));

        Ok(())
    }
}
