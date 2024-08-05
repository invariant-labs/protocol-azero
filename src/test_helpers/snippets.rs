#[macro_export]
macro_rules! create_dex {
    ($client:ident, $protocol_fee:expr) => {{
        let mut constructor = InvariantRef::new($protocol_fee);
        $client
            .instantiate("invariant", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed")
    }};
}

#[macro_export]
macro_rules! create_tokens {
    ($client:ident, $token_x_supply:expr, $token_y_supply:expr) => {{
        let mut token_x_constructor = TokenRef::new($token_x_supply, None, None, 0);
        let mut token_y_constructor = TokenRef::new($token_y_supply, None, None, 0);

        let token_x = $client
            .instantiate("token", &ink_e2e::alice(), &mut token_x_constructor)
            .submit()
            .await
            .expect("token x new failed");
        let token_y = $client
            .instantiate("token", &ink_e2e::alice(), &mut token_y_constructor)
            .submit()
            .await
            .expect("token y new failed");

        if token_x.account_id < token_y.account_id {
            (token_x, token_y)
        } else {
            (token_y, token_x)
        }
    }};
}

#[macro_export]
macro_rules! create_3_tokens {
    ($client:ident, $token_x_supply:expr, $token_y_supply:expr, $token_z_supply:expr) => {{
        let mut token_x_constructor = TokenRef::new($token_x_supply, None, None, 0);
        let mut token_y_constructor = TokenRef::new($token_y_supply, None, None, 0);
        let mut token_z_constructor = TokenRef::new($token_z_supply, None, None, 0);

        let token_x = $client
            .instantiate("token", &ink_e2e::alice(), &mut token_x_constructor)
            .submit()
            .await
            .expect("token x new failed");
        let token_y = $client
            .instantiate("token", &ink_e2e::alice(), &mut token_y_constructor)
            .submit()
            .await
            .expect("token y new failed");
        let token_z = $client
            .instantiate("token", &ink_e2e::alice(), &mut token_y_constructor)
            .submit()
            .await
            .expect("token z new failed");

        if (token_x.account_id < token_y.account_id) && (token_y.account_id < token_z.account_id) {
            (token_x, token_y, token_z)
        } else if (token_x.account_id < token_z.account_id)
            && (token_z.account_id < token_y.account_id)
        {
            (token_x, token_z, token_y)
        } else if (token_y.account_id < token_x.account_id)
            && (token_x.account_id < token_z.account_id)
        {
            (token_y, token_x, token_z)
        } else if (token_y.account_id < token_z.account_id)
            && (token_z.account_id < token_x.account_id)
        {
            (token_y, token_z, token_x)
        } else if (token_z.account_id < token_x.account_id)
            && (token_x.account_id < token_y.account_id)
        {
            (token_z, token_x, token_y)
        } else {
            (token_z, token_y, token_x)
        }
    }};
}

#[macro_export]
macro_rules! init_dex_and_tokens {
    ($client:ident) => {{
        let mint_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!($client, mint_amount, mint_amount);

        let protocol_fee = Percentage::from_scale(1, 2);
        let dex = create_dex!($client, protocol_fee);
        (dex, token_x, token_y)
    }};
}

#[macro_export]
macro_rules! init_dex_and_3_tokens {
    ($client:ident) => {{
        let mint_amount = u64::MAX as u128;
        let protocol_fee = Percentage::from_scale(1, 2);

        let dex = create_dex!($client, protocol_fee);
        let (token_x, token_y, token_z) =
            create_3_tokens!($client, mint_amount, mint_amount, mint_amount);

        (dex, token_x, token_y, token_z)
    }};
}

#[macro_export]
macro_rules! init_dex_and_tokens_max_mint_amount {
    ($client:ident) => {{
        let mint_amount = u128::MAX;
        let (token_x, token_y) = create_tokens!($client, mint_amount, mint_amount);

        let protocol_fee = Percentage::from_scale(1, 2);
        let dex = create_dex!($client, protocol_fee);
        (dex, token_x, token_y)
    }};
}

#[macro_export]
macro_rules! init_slippage_dex_and_tokens {
    ($client:ident) => {{
        let mint_amount = 10u128.pow(23);
        let (token_x, token_y) = create_tokens!($client, mint_amount, mint_amount);

        let protocol_fee = Percentage::from_scale(1, 2);
        let dex = create_dex!($client, protocol_fee);
        (dex, token_x, token_y)
    }};
}

#[macro_export]
macro_rules! create_standard_fee_tiers {
    ($client:ident, $dex:ty, $dex_address:expr) => {{
        // 1 * 10^(-4) = 0.0001 = 0.01%
        let caller = ink_e2e::alice();
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(1, 4), 1).unwrap(),
            caller
        );
        // 5 * 10^(-4) = 0.0005 = 0.05%
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(5, 4), 5).unwrap(),
            caller
        );
        // 1  * 10^(-3) = 0.001 = 0.1%
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(1, 3), 10).unwrap(),
            caller
        );
        // 3 * 10(-3) = 0.003 = 0.3%
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(3, 3), 30).unwrap(),
            caller
        );
        // 1 * 10^(-2) = 0.01 = 1%
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(1, 2), 100).unwrap(),
            caller
        );
        // 5 * 10^(-2) = 0.05 = 5%
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(5, 2), 100).unwrap(),
            caller
        );
        // 1 * 10^(-1) = 0.1 = 10%
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(1, 1), 100).unwrap(),
            caller
        );
        // 5 * 10^(-1) = 0.5 = 50%
        add_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap(),
            caller
        );
    }};
}

#[macro_export]
macro_rules! init_basic_pool {
    ($client:ident, $dex:ident, $token_x:ident, $token_y:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();
        add_fee_tier!($client, $dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();
    }};
}

#[macro_export]
macro_rules! init_slippage_pool_with_liquidity {
    ($client:ident, $dex:ident, $token_x:ident, $token_y:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();
        add_fee_tier!($client, $dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();

        let mint_amount = 10u128.pow(10);
        approve!($client, $token_x, $dex.account_id, mint_amount, alice).unwrap();
        approve!($client, $token_y, $dex.account_id, mint_amount, alice).unwrap();

        let pool_key = PoolKey::new($token_x.account_id, $token_y.account_id, fee_tier).unwrap();
        let lower_tick = -1000;
        let upper_tick = 1000;
        let liquidity = Liquidity::from_integer(10_000_000_000u128);

        let pool_before = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();
        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;
        create_position!(
            $client,
            $dex,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        let pool_after = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();

        assert_eq!(pool_after.liquidity, liquidity);

        pool_key
    }};
}

#[macro_export]
macro_rules! init_basic_position {
    ($client:ident, $dex:ident, $token_x:ident, $token_y:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();

        let mint_amount = 10u128.pow(10);
        approve!($client, $token_x, $dex.account_id, mint_amount, alice).unwrap();
        approve!($client, $token_y, $dex.account_id, mint_amount, alice).unwrap();

        let pool_key = PoolKey::new($token_x.account_id, $token_y.account_id, fee_tier).unwrap();
        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();
        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;
        create_position!(
            $client,
            $dex,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        let pool_after = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();

        assert_eq!(pool_after.liquidity, liquidity);
    }};
}

#[macro_export]
macro_rules! init_cross_position {
    ($client:ident, $dex:ident, $token_x:ident, $token_y:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();

        let mint_amount = 10u128.pow(10);
        approve!($client, $token_x, $dex.account_id, mint_amount, alice).unwrap();
        approve!($client, $token_y, $dex.account_id, mint_amount, alice).unwrap();

        let pool_key = PoolKey::new($token_x.account_id, $token_y.account_id, fee_tier).unwrap();
        let lower_tick = -40;
        let upper_tick = -10;
        let liquidity = Liquidity::from_integer(1000000);

        let pool_before = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();
        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;
        create_position!(
            $client,
            $dex,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        )
        .unwrap();

        let pool_after = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();

        assert_eq!(pool_after.liquidity, pool_before.liquidity);
    }};
}

#[macro_export]
macro_rules! init_basic_swap {
    ($client:ident, $dex:ident, $token_x:ident, $token_y:ident) => {{
        let fee = Percentage::from_scale(6, 3);
        let tick_spacing = 10;
        let fee_tier = FeeTier { fee, tick_spacing };
        let pool_key = PoolKey::new($token_x.account_id, $token_y.account_id, fee_tier).unwrap();
        let lower_tick = -20;

        let amount = 1000;
        let bob = ink_e2e::bob();
        mint!($client, $token_x, address_of!(Bob), amount, bob).unwrap();
        let amount_x = balance_of!($client, $token_x, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!($client, $token_x, $dex.account_id, amount, bob).unwrap();

        let amount_x = balance_of!($client, $token_x, $dex.account_id);
        let amount_y = balance_of!($client, $token_y, $dex.account_id);
        assert_eq!(amount_x, 500);
        assert_eq!(amount_y, 1000);

        let pool_before = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            pool_key.fee_tier
        )
        .unwrap();

        let swap_amount = TokenAmount::new(amount);
        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        swap!(
            $client,
            $dex,
            pool_key,
            true,
            swap_amount,
            true,
            slippage,
            bob
        )
        .unwrap();

        let pool_after = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();
        assert_eq!(pool_after.liquidity, pool_before.liquidity);
        assert_eq!(pool_after.current_tick_index, lower_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = balance_of!($client, $token_x, address_of!(Bob));
        let amount_y = balance_of!($client, $token_y, address_of!(Bob));
        assert_eq!(amount_x, 0);
        assert_eq!(amount_y, 993);

        let amount_x = balance_of!($client, $token_x, $dex.account_id);
        let amount_y = balance_of!($client, $token_y, $dex.account_id);
        assert_eq!(amount_x, 1500);
        assert_eq!(amount_y, 7);

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(50000000000000000000000_u128.into())
        );
        assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0.into()));

        assert_eq!(pool_after.fee_protocol_token_x, TokenAmount::new(1));
        assert_eq!(pool_after.fee_protocol_token_y, TokenAmount::new(0));
    }};
}

#[macro_export]
macro_rules! init_cross_swap {
    ($client:ident, $dex:ident, $token_x:ident, $token_y:ident) => {{
        let fee = Percentage::from_scale(6, 3);
        let tick_spacing = 10;
        let fee_tier = FeeTier { fee, tick_spacing };
        let pool_key = PoolKey::new($token_x.account_id, $token_y.account_id, fee_tier).unwrap();
        let lower_tick = -20;

        let amount = 1000;
        let bob = ink_e2e::bob();
        mint!($client, $token_x, address_of!(Bob), amount, bob).unwrap();
        let amount_x = balance_of!($client, $token_x, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!($client, $token_x, $dex.account_id, amount, bob).unwrap();

        let amount_x = balance_of!($client, $token_x, $dex.account_id);
        let amount_y = balance_of!($client, $token_y, $dex.account_id);
        assert_eq!(amount_x, 500);
        assert_eq!(amount_y, 2499);

        let pool_before = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();

        let swap_amount = TokenAmount::new(amount);
        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        swap!(
            $client,
            $dex,
            pool_key,
            true,
            swap_amount,
            true,
            slippage,
            bob
        )
        .unwrap();

        let pool_after = get_pool!(
            $client,
            $dex,
            $token_x.account_id,
            $token_y.account_id,
            fee_tier
        )
        .unwrap();
        let position_liquidity = Liquidity::from_integer(1000000);
        assert_eq!(
            pool_after.liquidity - position_liquidity,
            pool_before.liquidity
        );
        assert_eq!(pool_after.current_tick_index, lower_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = balance_of!($client, $token_x, address_of!(Bob));
        let amount_y = balance_of!($client, $token_y, address_of!(Bob));
        assert_eq!(amount_x, 0);
        assert_eq!(amount_y, 990);

        let amount_x = balance_of!($client, $token_x, $dex.account_id);
        let amount_y = balance_of!($client, $token_y, $dex.account_id);
        assert_eq!(amount_x, 1500);
        assert_eq!(amount_y, 1509);

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(40000000000000000000000_u128.into())
        );
        assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0.into()));

        assert_eq!(pool_after.fee_protocol_token_x, TokenAmount::new(2));
        assert_eq!(pool_after.fee_protocol_token_y, TokenAmount::new(0));
    }};
}

#[macro_export]
macro_rules! swap_exact_limit {
    ($client:ident, $dex:ident, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $caller:ident) => {{
        let sqrt_price_limit = if $x_to_y {
            SqrtPrice::new(MIN_SQRT_PRICE)
        } else {
            SqrtPrice::new(MAX_SQRT_PRICE)
        };

        let quote_result = quote!(
            $client,
            $dex,
            $pool_key,
            $x_to_y,
            $amount,
            $by_amount_in,
            sqrt_price_limit
        )
        .unwrap();
        swap!(
            $client,
            $dex,
            $pool_key,
            $x_to_y,
            $amount,
            $by_amount_in,
            quote_result.target_sqrt_price,
            $caller
        )
        .unwrap();
    }};
}

#[macro_export]
macro_rules! big_deposit_and_swap {
    ($client:ident, $x_to_y:expr) => {{
        let (dex, token_x, token_y) = init_dex_and_tokens_max_mint_amount!($client);

        let mint_amount = 2u128.pow(73) - 1;
        let alice = ink_e2e::alice();
        approve!($client, token_x, dex.account_id, u128::MAX, alice).unwrap();
        approve!($client, token_y, dex.account_id, u128::MAX, alice).unwrap();

        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 1,
        };
        add_fee_tier!($client, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            $client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        let lower_tick = if $x_to_y {
            -(fee_tier.tick_spacing as i32)
        } else {
            0
        };
        let upper_tick = if $x_to_y {
            0
        } else {
            fee_tier.tick_spacing as i32
        };
        let pool = get_pool!(
            $client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();

        let liquidity_delta = if $x_to_y {
            get_liquidity_by_y(
                TokenAmount(mint_amount),
                lower_tick,
                upper_tick,
                pool.sqrt_price,
                true,
            )
            .unwrap()
            .l
        } else {
            get_liquidity_by_x(
                TokenAmount(mint_amount),
                lower_tick,
                upper_tick,
                pool.sqrt_price,
                true,
            )
            .unwrap()
            .l
        };

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;
        create_position!(
            $client,
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

        let amount_x = balance_of!($client, token_x, address_of!(Alice));
        let amount_y = balance_of!($client, token_y, address_of!(Alice));
        if $x_to_y {
            assert_eq!(amount_x, 340282366920938463463374607431768211455);
            assert_eq!(amount_y, 340282366920938454018641641692477784064);
        } else {
            assert_eq!(amount_x, 340282366920938454018641641692477784064);
            assert_eq!(amount_y, 340282366920938463463374607431768211455);
        }

        let sqrt_price_limit = if $x_to_y {
            SqrtPrice::new(MIN_SQRT_PRICE)
        } else {
            SqrtPrice::new(MAX_SQRT_PRICE)
        };

        swap!(
            $client,
            dex,
            pool_key,
            $x_to_y,
            TokenAmount(mint_amount),
            true,
            sqrt_price_limit,
            alice
        )
        .unwrap();

        let amount_x = balance_of!($client, token_x, address_of!(Alice));
        let amount_y = balance_of!($client, token_y, address_of!(Alice));
        if $x_to_y {
            assert_eq!(amount_x, 340282366920938454018641641692477784064);
            assert_ne!(amount_y, 0);
        } else {
            assert_ne!(amount_x, 0);
            assert_eq!(amount_y, 340282366920938454018641641692477784064);
        }
    }};
}

#[macro_export]
macro_rules! multiple_swap {
    ($client:ident, $x_to_y:expr) => {{
        let (dex, token_x, token_y) = init_dex_and_tokens!($client);

        let fee_tier = FeeTier {
            fee: Percentage::from_scale(1, 3),
            tick_spacing: 1,
        };
        let alice = ink_e2e::alice();
        add_fee_tier!($client, dex, fee_tier, alice).unwrap();

        let init_tick = 0;
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
        create_pool!(
            $client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier,
            init_sqrt_price,
            init_tick,
            alice
        )
        .unwrap();

        let mint_amount = 10u128.pow(10);
        approve!($client, token_x, dex.account_id, mint_amount, alice).unwrap();
        approve!($client, token_y, dex.account_id, mint_amount, alice).unwrap();

        let pool_key = PoolKey::new(token_x.account_id, token_y.account_id, fee_tier).unwrap();
        let mut upper_tick = 953;
        let mut lower_tick = -upper_tick;

        let amount = 100;
        let pool_data = get_pool!(
            $client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        let result = get_liquidity(
            TokenAmount(amount),
            TokenAmount(amount),
            lower_tick,
            upper_tick,
            pool_data.sqrt_price,
            true,
        )
        .unwrap();
        let _amount_x = result.x;
        let _amount_y = result.y;
        let liquidity_delta = result.l;
        let slippage_limit_lower = pool_data.sqrt_price;
        let slippage_limit_upper = pool_data.sqrt_price;

        create_position!(
            $client,
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

        let bob = ink_e2e::bob();
        if $x_to_y {
            mint!($client, token_x, address_of!(Bob), amount, bob).unwrap();
            let amount_x = balance_of!($client, token_x, address_of!(Bob));
            assert_eq!(amount_x, amount);
            approve!($client, token_x, dex.account_id, amount, bob).unwrap();
        } else {
            mint!($client, token_y, address_of!(Bob), amount, bob).unwrap();
            let amount_y = balance_of!($client, token_y, address_of!(Bob));
            assert_eq!(amount_y, amount);
            approve!($client, token_y, dex.account_id, amount, bob).unwrap();
        }

        let swap_amount = TokenAmount(10);
        for i in 1..=10 {
            swap_exact_limit!($client, dex, pool_key, $x_to_y, swap_amount, true, bob);
        }

        let pool = get_pool!(
            $client,
            dex,
            token_x.account_id,
            token_y.account_id,
            fee_tier
        )
        .unwrap();
        if $x_to_y {
            assert_eq!(pool.current_tick_index, -821);
        } else {
            assert_eq!(pool.current_tick_index, 820);
        }
        assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(0.into()));
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0.into()));
        if $x_to_y {
            assert_eq!(pool.fee_protocol_token_x, TokenAmount(10));
            assert_eq!(pool.fee_protocol_token_y, TokenAmount(0));
        } else {
            assert_eq!(pool.fee_protocol_token_x, TokenAmount(0));
            assert_eq!(pool.fee_protocol_token_y, TokenAmount(10));
        }
        assert_eq!(pool.liquidity, liquidity_delta);
        if $x_to_y {
            assert_eq!(pool.sqrt_price, SqrtPrice::new(959805958620596146276151));
        } else {
            assert_eq!(pool.sqrt_price, SqrtPrice::new(1041877257604411525269920));
        }

        let dex_amount_x = balance_of!($client, token_x, dex.account_id);
        let dex_amount_y = balance_of!($client, token_y, dex.account_id);
        if $x_to_y {
            assert_eq!(dex_amount_x, 200);
            assert_eq!(dex_amount_y, 20);
        } else {
            assert_eq!(dex_amount_x, 20);
            assert_eq!(dex_amount_y, 200);
        }

        let user_amount_x = balance_of!($client, token_x, address_of!(Bob));
        let user_amount_y = balance_of!($client, token_y, address_of!(Bob));
        if $x_to_y {
            assert_eq!(user_amount_x, 0);
            assert_eq!(user_amount_y, 80);
        } else {
            assert_eq!(user_amount_x, 80);
            assert_eq!(user_amount_y, 0);
        }
    }};
}

#[macro_export]
macro_rules! positions_equals {
    ($a:expr, $b:expr) => {{
        assert_eq!($a.fee_growth_inside_x, $b.fee_growth_inside_x);
        assert_eq!($a.fee_growth_inside_y, $b.fee_growth_inside_y);
        assert_eq!($a.liquidity, $b.liquidity);
        assert_eq!($a.lower_tick_index, $b.lower_tick_index);
        assert_eq!($a.upper_tick_index, $b.upper_tick_index);
        assert_eq!($a.pool_key, $b.pool_key);
        assert_eq!($a.tokens_owed_x, $b.tokens_owed_x);
        assert_eq!($a.tokens_owed_y, $b.tokens_owed_y);
        assert_eq!($a.created_at, $b.created_at);
    }};
}

#[macro_export]
macro_rules! liquidity_tick_equals {
    ($a:expr, $b:expr) => {{
        assert_eq!($a.index, $b.index);
        assert_eq!($a.liquidity_change, $b.liquidity_change);
        assert_eq!($a.sign, $b.sign);
    }};
}
