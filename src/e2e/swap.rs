#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{entrypoints::InvariantTrait, FeeTier, InvariantError, PoolKey},
        invariant::InvariantRef,
        math::{
            types::{
                fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage,
                sqrt_price::calculate_sqrt_price, sqrt_price::SqrtPrice, token_amount::TokenAmount,
            },
            MAX_SQRT_PRICE, MIN_SQRT_PRICE,
        },
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, create_dex, create_pool, create_position,
        create_tokens, get_pool, get_tick, init_basic_pool, init_basic_position, init_basic_swap,
        init_dex_and_tokens, is_tick_initialized, mint, quote, swap,
    };
    use token::PSP22Mintable;
    use token::Token;
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn swap(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens!(client);
        init_basic_pool!(client, dex, token_x, token_y);
        init_basic_position!(client, dex, token_x, token_y);
        init_basic_swap!(client, dex, token_x, token_y);
        Ok(())
    }

    #[ink_e2e::test]
    async fn test_swap_x_to_y(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(6, 3));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();
        let bob = ink_e2e::bob();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

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

        let lower_tick_index = -20;
        let middle_tick_index = -10;
        let upper_tick_index = 10;

        let liquidity_delta = Liquidity::from_integer(1000000);

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index - 20,
            middle_tick_index,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
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

        assert_eq!(pool.liquidity, liquidity_delta);

        let amount = 1000;
        let swap_amount = TokenAmount(amount);

        mint!(client, token_x, address_of!(Bob), amount, bob).unwrap();
        approve!(client, token_x, dex.account_id, amount, bob).unwrap();

        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        let target_sqrt_price = quote!(client, dex, pool_key, true, swap_amount, true, slippage)
            .unwrap()
            .target_sqrt_price;

        let before_dex_x = balance_of!(client, token_x, dex.account_id);
        let before_dex_y = balance_of!(client, token_y, dex.account_id);

        swap!(
            client,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            target_sqrt_price,
            bob
        )
        .unwrap();

        // Load states
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, lower_tick_index).unwrap();
        let middle_tick = get_tick!(client, dex, pool_key, middle_tick_index).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, upper_tick_index).unwrap();
        let lower_tick_bit = is_tick_initialized!(client, dex, pool_key, lower_tick_index);
        let middle_tick_bit = is_tick_initialized!(client, dex, pool_key, middle_tick_index);
        let upper_tick_bit = is_tick_initialized!(client, dex, pool_key, upper_tick_index);
        let bob_x = balance_of!(client, token_x, address_of!(Bob));
        let bob_y = balance_of!(client, token_y, address_of!(Bob));
        let dex_x = balance_of!(client, token_x, dex.account_id);
        let dex_y = balance_of!(client, token_y, dex.account_id);
        let delta_dex_y = before_dex_y - dex_y;
        let delta_dex_x = dex_x - before_dex_x;
        let expected_y = amount - 10;
        let expected_x = 0;

        // Check balances
        assert_eq!(bob_x, expected_x);
        assert_eq!(bob_y, expected_y);
        assert_eq!(delta_dex_x, amount);
        assert_eq!(delta_dex_y, expected_y);

        // Check Pool
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0.into()));
        assert_eq!(
            pool.fee_growth_global_x,
            FeeGrowth::new(40000000000000000000000_u128.into())
        );
        assert_eq!(pool.fee_protocol_token_y, TokenAmount(0));
        assert_eq!(pool.fee_protocol_token_x, TokenAmount(2));

        // Check Ticks
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(middle_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.fee_growth_outside_x, FeeGrowth::new(0.into()));
        assert_eq!(
            middle_tick.fee_growth_outside_x,
            FeeGrowth::new(30000000000000000000000_u128.into())
        );
        assert_eq!(lower_tick.fee_growth_outside_x, FeeGrowth::new(0.into()));
        assert!(lower_tick_bit);
        assert!(middle_tick_bit);
        assert!(upper_tick_bit);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_swap_y_to_x(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(6, 3));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();
        let bob = ink_e2e::bob();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

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

        let lower_tick_index = -10;
        let middle_tick_index = 10;
        let upper_tick_index = 20;

        let liquidity_delta = Liquidity::from_integer(1000000);

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            middle_tick_index,
            upper_tick_index + 20,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
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

        assert_eq!(pool.liquidity, liquidity_delta);

        let amount = 1000;
        let swap_amount = TokenAmount(amount);

        mint!(client, token_y, address_of!(Bob), amount, bob).unwrap();
        approve!(client, token_y, dex.account_id, amount, bob).unwrap();

        let target_sqrt_price = SqrtPrice::new(MAX_SQRT_PRICE);

        let target_sqrt_price = quote!(
            client,
            dex,
            pool_key,
            false,
            swap_amount,
            true,
            target_sqrt_price
        )
        .unwrap()
        .target_sqrt_price;

        let before_dex_x = balance_of!(client, token_x, dex.account_id);
        let before_dex_y = balance_of!(client, token_y, dex.account_id);

        swap!(
            client,
            dex,
            pool_key,
            false,
            swap_amount,
            true,
            target_sqrt_price,
            bob
        )
        .unwrap();

        // Load states
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let lower_tick = get_tick!(client, dex, pool_key, lower_tick_index).unwrap();
        let middle_tick = get_tick!(client, dex, pool_key, middle_tick_index).unwrap();
        let upper_tick = get_tick!(client, dex, pool_key, upper_tick_index).unwrap();
        let lower_tick_bit = is_tick_initialized!(client, dex, pool_key, lower_tick_index);
        let middle_tick_bit = is_tick_initialized!(client, dex, pool_key, middle_tick_index);
        let upper_tick_bit = is_tick_initialized!(client, dex, pool_key, upper_tick_index);
        let bob_x = balance_of!(client, token_x, address_of!(Bob));
        let bob_y = balance_of!(client, token_y, address_of!(Bob));
        let dex_x = balance_of!(client, token_x, dex.account_id);
        let dex_y = balance_of!(client, token_y, dex.account_id);
        let delta_dex_x = before_dex_x - dex_x;
        let delta_dex_y = dex_y - before_dex_y;
        let expected_x = amount - 10;
        let expected_y = 0;

        // Check balances
        assert_eq!(bob_x, expected_x);
        assert_eq!(bob_y, expected_y);
        assert_eq!(delta_dex_x, expected_x);
        assert_eq!(delta_dex_y, amount);

        // Check Pool
        assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(0.into()));
        assert_eq!(
            pool.fee_growth_global_y,
            FeeGrowth::new(40000000000000000000000_u128.into())
        );
        assert_eq!(pool.fee_protocol_token_x, TokenAmount(0));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount(2));

        // Check Ticks
        assert_eq!(lower_tick.liquidity_change, liquidity_delta);
        assert_eq!(middle_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.liquidity_change, liquidity_delta);
        assert_eq!(upper_tick.fee_growth_outside_y, FeeGrowth::new(0.into()));
        assert_eq!(
            middle_tick.fee_growth_outside_y,
            FeeGrowth::new(30000000000000000000000_u128.into())
        );
        assert_eq!(lower_tick.fee_growth_outside_y, FeeGrowth::new(0.into()));
        assert!(lower_tick_bit);
        assert!(middle_tick_bit);
        assert!(upper_tick_bit);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_swap_not_enough_liquidity_token_x(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(6, 3));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

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

        let lower_tick_index = -10;
        let middle_tick_index = 10;
        let upper_tick_index = 20;

        let liquidity_delta = Liquidity::from_integer(1000000);

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            middle_tick_index,
            upper_tick_index + 20,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
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

        assert_eq!(pool.liquidity, liquidity_delta);

        let amount = 1000;
        let swap_amount = TokenAmount(amount);
        let bob = ink_e2e::bob();
        mint!(client, token_x, address_of!(Bob), amount, alice).unwrap();
        approve!(client, token_x, dex.account_id, amount, bob).unwrap();

        let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

        let error = swap!(
            client,
            dex,
            pool_key,
            true,
            swap_amount,
            true,
            target_sqrt_price,
            bob
        );

        assert_eq!(error, Err(InvariantError::TickLimitReached));

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_swap_not_enough_liquidity_token_y(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let dex = create_dex!(client, Percentage::from_scale(6, 3));
        let initial_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!(client, initial_amount, initial_amount);

        let alice = ink_e2e::alice();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

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

        let lower_tick_index = -20;
        let middle_tick_index = -10;
        let upper_tick_index = 10;

        let liquidity_delta = Liquidity::from_integer(1000000);

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
            alice
        )
        .unwrap();

        create_position!(
            client,
            dex,
            pool_key,
            lower_tick_index - 20,
            middle_tick_index,
            liquidity_delta,
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
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

        assert_eq!(pool.liquidity, liquidity_delta);

        let amount = 1000;
        let swap_amount = TokenAmount(amount);
        let bob = ink_e2e::bob();
        mint!(client, token_y, address_of!(Bob), amount, alice).unwrap();
        approve!(client, token_y, dex.account_id, amount, bob).unwrap();

        let slippage = SqrtPrice::new(MAX_SQRT_PRICE);

        let res = swap!(
            client,
            dex,
            pool_key,
            false,
            swap_amount,
            true,
            slippage,
            bob
        );

        assert_eq!(res, Err(InvariantError::TickLimitReached));

        Ok(())
    }
}
