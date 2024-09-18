#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, InvariantError, PoolKey},
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
    use ink::primitives::AccountId;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, get_position, get_tick, is_tick_initialized, mint,
        remove_position, swap,
    };
    use token::PSP22Mintable;
    use token::Token;
    use token::TokenRef;
    use token::PSP22;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_create_position(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_position_same_upper_and_lower_tick(
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

        let result = create_position!(
            client,
            dex,
            pool_key,
            10,
            10,
            Liquidity::new(10),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        );

        assert_eq!(result, Err(InvariantError::InvalidTickIndex));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_remove_position(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
        let alice = ink_e2e::alice();
        let bob = ink_e2e::bob();
        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let remove_position_index = 0;

        let initial_mint = 10u128.pow(10);

        let dex = create_dex!(client, Percentage::from_scale(1, 2));
        let (token_x, token_y) = create_tokens!(client, initial_mint, initial_mint);

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();

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

        let lower_tick_index = -20;
        let upper_tick_index = 10;
        let liquidity_delta = Liquidity::from_integer(1_000_000);

        approve!(client, token_x, dex.account_id, initial_mint, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_mint, alice).unwrap();

        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            pool_state.sqrt_price,
            pool_state.sqrt_price,
            alice
        )
        .unwrap();

        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        assert_eq!(pool_state.liquidity, liquidity_delta);

        let liquidity_delta = Liquidity::new(liquidity_delta.get() * 1_000_000);
        {
            let incorrect_lower_tick_index = lower_tick_index - 50;
            let incorrect_upper_tick_index = upper_tick_index + 50;

            approve!(
                client,
                token_x,
                dex.account_id,
                liquidity_delta.get(),
                alice
            )
            .unwrap();
            approve!(
                client,
                token_y,
                dex.account_id,
                liquidity_delta.get(),
                alice
            )
            .unwrap();

            create_position!(
                client,
                dex,
                pool_key,
                incorrect_lower_tick_index,
                incorrect_upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            )
            .unwrap();

            let position_state = get_position!(client, dex, 1, alice).unwrap();
            // Check position
            assert!(position_state.lower_tick_index == incorrect_lower_tick_index);
            assert!(position_state.upper_tick_index == incorrect_upper_tick_index);
        }

        let amount = 1000;
        mint!(client, token_x, address_of!(Bob), amount, bob).unwrap();
        let amount_x = balance_of!(client, token_x, address_of!(Bob));
        assert_eq!(amount_x, amount);

        approve!(client, token_x, dex.account_id, amount, bob).unwrap();

        let pool_state_before = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let swap_amount = TokenAmount::new(amount);
        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        swap!(
            client,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            slippage,
            bob
        )
        .unwrap();

        let pool_state_after = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(
            pool_state_after.fee_growth_global_x,
            FeeGrowth::new(49999950000049999_u128.into())
        );
        assert_eq!(pool_state_after.fee_protocol_token_x, TokenAmount(1));
        assert_eq!(pool_state_after.fee_protocol_token_y, TokenAmount(0));

        assert!(pool_state_after
            .sqrt_price
            .lt(&pool_state_before.sqrt_price));

        assert_eq!(pool_state_after.liquidity, pool_state_before.liquidity);
        assert_eq!(pool_state_after.current_tick_index, -10);
        assert_ne!(pool_state_after.sqrt_price, pool_state_before.sqrt_price);

        let amount_x = balance_of!(client, token_x, address_of!(Bob));
        let amount_y = balance_of!(client, token_y, address_of!(Bob));
        assert_eq!(amount_x, 0);
        assert_eq!(amount_y, 993);

        // pre load dex balances
        let dex_x_before_remove = balance_of!(client, token_x, dex.account_id);
        let dex_y_before_remove = balance_of!(client, token_y, dex.account_id);

        // Remove position
        remove_position!(client, dex, remove_position_index, alice).unwrap();

        // Load states
        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, lower_tick_index);
        let upper_tick = get_tick!(client, dex, pool_key, upper_tick_index);
        let lower_tick_bit = is_tick_initialized!(client, dex, pool_key, lower_tick_index);
        let upper_tick_bit = is_tick_initialized!(client, dex, pool_key, upper_tick_index);
        let dex_x = balance_of!(client, token_x, dex.account_id);
        let dex_y = balance_of!(client, token_y, dex.account_id);
        let expected_withdrawn_x = 499;
        let expected_withdrawn_y = 999;
        let expected_fee_x = 0;

        assert_eq!(
            dex_x_before_remove - dex_x,
            expected_withdrawn_x + expected_fee_x
        );
        assert_eq!(dex_y_before_remove - dex_y, expected_withdrawn_y);

        // Check ticks
        assert_eq!(lower_tick, Err(InvariantError::TickNotFound));
        assert_eq!(upper_tick, Err(InvariantError::TickNotFound));

        // Check tickmap
        assert!(!lower_tick_bit);
        assert!(!upper_tick_bit);

        // Check pool
        assert!(pool_state.liquidity == liquidity_delta);
        assert!(pool_state.current_tick_index == -10);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_position_within_current_tick(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let max_tick_test = 177_450; // for tickSpacing 4
        let min_tick_test = -max_tick_test;
        let alice = ink_e2e::alice();
        let init_tick = -23028;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let dex = create_dex!(client, Percentage::new(0));
        let initial_balance = 100_000_000;

        let (token_x, token_y) = create_tokens!(client, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

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

        approve!(client, token_x, dex.account_id, initial_balance, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let lower_tick_index = min_tick_test + 10;
        let upper_tick_index = max_tick_test - 10;

        let liquidity_delta = Liquidity::from_integer(100);

        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            pool_state.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        // Load states
        let position_state = get_position!(client, dex, 0, alice).unwrap();
        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, lower_tick_index).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, upper_tick_index).unwrap();
        let alice_x = balance_of!(client, token_x, address_of!(Alice));
        let alice_y = balance_of!(client, token_y, address_of!(Alice));
        let dex_x = balance_of!(client, token_x, dex.account_id);
        let dex_y = balance_of!(client, token_y, dex.account_id);

        let zero_fee = FeeGrowth::new(0.into());
        let expected_x_increase = 317;
        let expected_y_increase = 32;

        // Check ticks
        assert!(lower_tick.index == lower_tick_index);
        assert!(upper_tick.index == upper_tick_index);
        assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
        assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert!(lower_tick.sign);
        assert!(!upper_tick.sign);

        // Check pool
        assert!(pool_state.liquidity == liquidity_delta);
        assert!(pool_state.current_tick_index == init_tick);

        // Check position
        assert!(position_state.pool_key == pool_key);
        assert!(position_state.liquidity == liquidity_delta);
        assert!(position_state.lower_tick_index == lower_tick_index);
        assert!(position_state.upper_tick_index == upper_tick_index);
        assert!(position_state.fee_growth_inside_x == zero_fee);
        assert!(position_state.fee_growth_inside_y == zero_fee);

        // Check balances
        assert_eq!(alice_x, initial_balance.checked_sub(dex_x).unwrap());
        assert_eq!(alice_y, initial_balance.checked_sub(dex_y).unwrap());
        assert_eq!(dex_x, expected_x_increase);
        assert_eq!(dex_y, expected_y_increase);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_position_below_current_tick(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let init_tick = -23028;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        let dex = create_dex!(client, Percentage::new(0));
        let initial_balance = 10_000_000_000;

        let (token_x, token_y) = create_tokens!(client, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

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

        approve!(client, token_x, dex.account_id, initial_balance, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let lower_tick_index = -46080;
        let upper_tick_index = -23040;

        let liquidity_delta = Liquidity::from_integer(10_000);

        let pool_state_before = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            pool_state_before.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        // Load states
        let position_state = get_position!(client, dex, 0, alice).unwrap();
        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, lower_tick_index).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, upper_tick_index).unwrap();
        let alice_x = balance_of!(client, token_x, address_of!(Alice));
        let alice_y = balance_of!(client, token_y, address_of!(Alice));
        let dex_x = balance_of!(client, token_x, dex.account_id);
        let dex_y = balance_of!(client, token_y, dex.account_id);

        let zero_fee = FeeGrowth::new(0.into());
        let expected_x_increase = 0;
        let expected_y_increase = 2162;

        // Check ticks
        assert!(lower_tick.index == lower_tick_index);
        assert!(upper_tick.index == upper_tick_index);
        assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
        assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert!(lower_tick.sign);
        assert!(!upper_tick.sign);

        // Check pool
        assert!(pool_state.liquidity == pool_state_before.liquidity);
        assert!(pool_state.current_tick_index == init_tick);

        // Check position
        assert!(position_state.pool_key == pool_key);
        assert!(position_state.liquidity == liquidity_delta);
        assert!(position_state.lower_tick_index == lower_tick_index);
        assert!(position_state.upper_tick_index == upper_tick_index);
        assert!(position_state.fee_growth_inside_x == zero_fee);
        assert!(position_state.fee_growth_inside_y == zero_fee);

        // Check balances
        assert_eq!(alice_x, initial_balance.checked_sub(dex_x).unwrap());
        assert_eq!(alice_y, initial_balance.checked_sub(dex_y).unwrap());

        assert_eq!(dex_x, expected_x_increase);
        assert_eq!(dex_y, expected_y_increase);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_position_above_current_tick(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let alice = ink_e2e::alice();
        let init_tick = -23028;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let dex = create_dex!(client, Percentage::new(0));
        let initial_balance = 10_000_000_000;

        let (token_x, token_y) = create_tokens!(client, initial_balance, initial_balance);

        let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

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

        approve!(client, token_x, dex.account_id, initial_balance, alice).unwrap();
        approve!(client, token_y, dex.account_id, initial_balance, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let lower_tick_index = -22980;
        let upper_tick_index = 0;
        let liquidity_delta = Liquidity::from_integer(10_000);

        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            pool_state.sqrt_price,
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        // Load states
        let position_state = get_position!(client, dex, 0, alice).unwrap();
        let pool_state = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, lower_tick_index).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, upper_tick_index).unwrap();
        let alice_x = balance_of!(client, token_x, address_of!(Alice));
        let alice_y = balance_of!(client, token_y, address_of!(Alice));
        let dex_x = balance_of!(client, token_x, dex.account_id);
        let dex_y = balance_of!(client, token_y, dex.account_id);

        let zero_fee = FeeGrowth::new(0.into());
        let expected_x_increase = 21549;
        let expected_y_increase = 0;

        // Check ticks
        assert!(lower_tick.index == lower_tick_index);
        assert!(upper_tick.index == upper_tick_index);
        assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
        assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert!(lower_tick.sign);
        assert!(!upper_tick.sign);

        // Check pool
        assert!(pool_state.liquidity == Liquidity::new(0));
        assert!(pool_state.current_tick_index == init_tick);

        // Check position
        assert!(position_state.pool_key == pool_key);
        assert!(position_state.liquidity == liquidity_delta);
        assert!(position_state.lower_tick_index == lower_tick_index);
        assert!(position_state.upper_tick_index == upper_tick_index);
        assert!(position_state.fee_growth_inside_x == zero_fee);
        assert!(position_state.fee_growth_inside_y == zero_fee);

        // Check balances
        assert_eq!(alice_x, initial_balance.checked_sub(dex_x).unwrap());
        assert_eq!(alice_y, initial_balance.checked_sub(dex_y).unwrap());

        assert_eq!(dex_x, expected_x_increase);
        assert_eq!(dex_y, expected_y_increase);

        Ok(())
    }
}
