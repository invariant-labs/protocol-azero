#[cfg(test)]
pub mod e2e_tests {
    use crate::invariant::Invariant;
    use crate::{
        contracts::{
            entrypoints::InvariantTrait,
            logic::math::{get_liquidity_by_x, get_liquidity_by_y},
            FeeTier, PoolKey,
        },
        invariant::InvariantRef,
        math::{
            clamm::get_delta_y,
            liquidity::Liquidity,
            types::{
                percentage::Percentage,
                sqrt_price::{calculate_sqrt_price, get_max_tick, SqrtPrice},
                token_amount::TokenAmount,
            },
            MAX_SQRT_PRICE, MAX_TICK, MIN_SQRT_PRICE,
        },
    };
    use decimal::*;
    use ink_e2e::ContractsBackend;
    use test_helpers::{
        add_fee_tier, address_of, approve, balance_of, big_deposit_and_swap, create_dex,
        create_pool, create_position, create_tokens, get_pool, init_dex_and_tokens_max_mint_amount,
        swap,
    };
    use token::Token;
    use token::{TokenRef, PSP22};

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_limits_big_deposit_x_and_swap_y(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        big_deposit_and_swap!(client, true);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_limits_big_deposit_y_and_swap_x(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        big_deposit_and_swap!(client, false);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_limits_big_deposit_both_tokens(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens_max_mint_amount!(client);

        let mint_amount = 2u128.pow(73) - 1;
        let alice = ink_e2e::alice();
        approve!(client, token_x, dex.account_id, u128::MAX, alice).unwrap();
        approve!(client, token_y, dex.account_id, u128::MAX, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();

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

        let lower_tick = -(fee_tier.tick_spacing as i32);
        let upper_tick = fee_tier.tick_spacing as i32;
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let liquidity_delta = get_liquidity_by_x(
            TokenAmount(mint_amount),
            lower_tick,
            upper_tick,
            pool.sqrt_price,
            false,
        )
        .unwrap()
        .l;
        let y = get_delta_y(
            calculate_sqrt_price(lower_tick).unwrap(),
            pool.sqrt_price,
            liquidity_delta,
            true,
        )
        .unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;
        create_position!(
            client,
            dex,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        let user_amount_x = balance_of!(client, token_x, address_of!(Alice));
        let user_amount_y = balance_of!(client, token_y, address_of!(Alice));
        assert_eq!(user_amount_x, u128::MAX - mint_amount);
        assert_eq!(user_amount_y, u128::MAX - y.get());

        let contract_amount_x = balance_of!(client, token_x, dex.account_id);
        let contract_amount_y = balance_of!(client, token_y, dex.account_id);
        assert_eq!(contract_amount_x, mint_amount);
        assert_eq!(contract_amount_y, y.get());

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_deposit_limits_at_upper_limit(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens_max_mint_amount!(client);

        let mint_amount = 2u128.pow(105) - 1;
        let alice = ink_e2e::alice();
        approve!(client, token_x, dex.account_id, u128::MAX, alice).unwrap();
        approve!(client, token_y, dex.account_id, u128::MAX, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();
        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = get_max_tick(1);
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

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.current_tick_index, init_tick);
        assert_eq!(pool.sqrt_price, calculate_sqrt_price(init_tick).unwrap());

        let position_amount = mint_amount - 1;

        let liquidity_delta = get_liquidity_by_y(
            TokenAmount(position_amount),
            0,
            MAX_TICK,
            pool.sqrt_price,
            false,
        )
        .unwrap()
        .l;

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;
        create_position!(
            client,
            dex,
            pool_key,
            0,
            MAX_TICK,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_limits_big_deposit_and_swaps(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens_max_mint_amount!(client);

        let mint_amount = 2u128.pow(74) - 1;
        let alice = ink_e2e::alice();
        approve!(client, token_x, dex.account_id, u128::MAX, alice).unwrap();
        approve!(client, token_y, dex.account_id, u128::MAX, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();
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

        let pos_amount = mint_amount / 2;
        let lower_tick = -(fee_tier.tick_spacing as i32);
        let upper_tick = fee_tier.tick_spacing as i32;
        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = get_liquidity_by_x(
            TokenAmount(pos_amount),
            lower_tick,
            upper_tick,
            pool.sqrt_price,
            false,
        )
        .unwrap()
        .l;

        let y = get_delta_y(
            calculate_sqrt_price(lower_tick).unwrap(),
            pool.sqrt_price,
            liquidity_delta,
            true,
        )
        .unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;
        create_position!(
            client,
            dex,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        let user_amount_x = balance_of!(client, token_x, address_of!(Alice));
        let user_amount_y = balance_of!(client, token_y, address_of!(Alice));
        assert_eq!(user_amount_x, u128::MAX - pos_amount);
        assert_eq!(user_amount_y, u128::MAX - y.get());

        let contract_amount_x = balance_of!(client, token_x, dex.account_id);
        let contract_amount_y = balance_of!(client, token_y, dex.account_id);
        assert_eq!(contract_amount_x, pos_amount);
        assert_eq!(contract_amount_y, y.get());

        let swap_amount = TokenAmount(mint_amount / 8);

        for i in 1..=4 {
            let (_, sqrt_price_limit) = if i % 2 == 0 {
                (true, SqrtPrice::new(MIN_SQRT_PRICE))
            } else {
                (false, SqrtPrice::new(MAX_SQRT_PRICE))
            };

            swap!(
                client,
                dex,
                pool_key,
                i % 2 == 0,
                swap_amount,
                true,
                sqrt_price_limit,
                alice
            )
            .unwrap();
        }

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_limits_full_range_with_max_liquidity(
        mut client: ink_e2e::Client<C, E>,
    ) -> E2EResult<()> {
        let (dex, token_x, token_y) = init_dex_and_tokens_max_mint_amount!(client);

        let alice = ink_e2e::alice();
        approve!(client, token_x, dex.account_id, u128::MAX, alice).unwrap();
        approve!(client, token_y, dex.account_id, u128::MAX, alice).unwrap();

        let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();
        add_fee_tier!(client, dex, fee_tier, alice).unwrap();

        let init_tick = get_max_tick(1);
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

        let pool = get_pool!(
            client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool.current_tick_index, init_tick);
        assert_eq!(pool.sqrt_price, calculate_sqrt_price(init_tick).unwrap());

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let liquidity_delta = Liquidity::new(1208899457432799883049625000361); // < 2^100
        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;
        create_position!(
            client,
            dex,
            pool_key,
            -MAX_TICK,
            MAX_TICK,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();
        let contract_amount_x = balance_of!(client, token_x, dex.account_id);
        let contract_amount_y = balance_of!(client, token_y, dex.account_id);

        let expected_x = 0;
        let expected_y = 340282366920938463463374607431721256973; // < 2^127
        assert_eq!(contract_amount_x, expected_x);
        assert_eq!(contract_amount_y, expected_y);
        Ok(())
    }
}
