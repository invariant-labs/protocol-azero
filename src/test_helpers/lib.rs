pub mod entrypoints;
pub mod token;

#[macro_export]
macro_rules! create_tokens {
    ($client:ident, $x:ty, $y:ty, $supply_x:expr, $supply_y:expr) => {{
        // ink_e2e client
        // x:ty  || y:ty => x token ref => TokenRef
        // supply_x:expr || supply_y:expr => amount of initial supply x => 100
        let constructor_x = <$x>::new($supply_x, None, None, 0);
        let constructor_y = <$y>::new($supply_y, None, None, 0);
        let x = $client
            .instantiate("token", &ink_e2e::alice(), constructor_x, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        let y = $client
            .instantiate("token", &ink_e2e::alice(), constructor_y, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        if x < y {
            (x, y)
        } else {
            (y, x)
        }
    }};
}

#[macro_export]
macro_rules! create_dex {
    ($client:ident,  $dex:ty, $dex_fee:expr) => {{
        // ink_e2e client
        // dex:ty => dex ref => ContractRef
        // dex_fee:exp => protocol_fee => Percentage::new(..)
        let constructor_dex = <$dex>::new($dex_fee);
        let dex = $client
            .instantiate("contract", &ink_e2e::alice(), constructor_dex, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        dex
    }};
}

#[macro_export]
macro_rules! create_standard_fee_tiers {
    ($client:ident, $dex:ty, $dex_address:expr) => {{
        // client => ink_e2e_client
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // 1 * 10^(-4) = 0.0001 = 0.01%
        let caller = ink_e2e::alice();
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 4),
                tick_spacing: 1
            },
            caller
        );
        // 5 * 10^(-4) = 0.0005 = 0.05%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(5, 4),
                tick_spacing: 5
            },
            caller
        );
        // 1  * 10^(-3) = 0.001 = 0.1%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 3),
                tick_spacing: 10
            },
            caller
        );
        // 3 * 10(-3) = 0.003 = 0.3%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(3, 3),
                tick_spacing: 30
            },
            caller
        );
        // 1 * 10^(-2) = 0.01 = 1%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 2),
                tick_spacing: 100
            },
            caller
        );
        // 5 * 10^(-2) = 0.05 = 5%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(5, 2),
                tick_spacing: 100
            },
            caller
        );
        // 1 * 10^(-1) = 0.1 = 10%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(1, 1),
                tick_spacing: 100
            },
            caller
        );
        // 5 * 10^(-1) = 0.5 = 50%
        create_fee_tier!(
            $client,
            $dex,
            $dex_address,
            FeeTier {
                fee: Percentage::from_scale(5, 1),
                tick_spacing: 100
            },
            caller
        );
    }};
}

#[macro_export]
macro_rules! create_fee_tier {
    ($client:ident, $dex:ty, $dex_address:expr, $fee_tier:expr, $caller:ident) => {{
        // client => ink_e2e_client
        // x:ident || y:ident => Addresses of x and y tokens
        // dex:ty => ContractRef
        // dex_address:expr => Address of contract
        // fee:expr => Percentage
        // spacing:expr => tick_spacing as u16
        let _msg = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.add_fee_tier($fee_tier));
        $client
            .call(&$caller, _msg, 0, None)
            .await
            .expect("Fee Tier creation failed")
            .return_value()
    }};
}

#[macro_export]
macro_rules! init_dex_and_tokens {
    ($client:ident, $dex:ty, $token:ty) => {{
        let mint_amount = 10u128.pow(10);
        let (token_x, token_y) = create_tokens!($client, $token, $token, mint_amount, mint_amount);

        let protocol_fee = Percentage::from_scale(1, 2);
        let dex = create_dex!($client, $dex, protocol_fee);
        (dex, token_x, token_y)
    }};
}

#[macro_export]
macro_rules! init_dex_and_3_tokens {
    ($client:ident, $dex:ty, $token:ty) => {{
        let mint_amount = u64::MAX as u128;
        let protocol_fee = Percentage::from_scale(1, 2);

        let dex = create_dex!($client, $dex, protocol_fee);
        let (token_x, token_y, token_z) = create_3_tokens!(
            $client,
            $token,
            $token,
            $token,
            mint_amount,
            mint_amount,
            mint_amount
        );

        (dex, token_x, token_y, token_z)
    }};
}

#[macro_export]
macro_rules! create_3_tokens {
    ($client:ident, $x:ty, $y:ty, $z:ty, $supply_x:expr, $supply_y:expr, $supply_z:expr) => {{
        let constructor_x = <$x>::new($supply_x, None, None, 0);
        let constructor_y = <$y>::new($supply_y, None, None, 0);
        let constructor_z = <$z>::new($supply_z, None, None, 0);

        let x = $client
            .instantiate("token", &ink_e2e::alice(), constructor_x, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        let y = $client
            .instantiate("token", &ink_e2e::alice(), constructor_y, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        let z = $client
            .instantiate("token", &ink_e2e::alice(), constructor_z, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;

        let mut v = [x, y, z];
        v.sort();
        let [x, y, z] = v;
        (x, y, z)
    }};
}

#[macro_export]
macro_rules! init_basic_pool {
    ($client:ident, $dex:ty, $token:ty, $dex_address:ident, $token_x_address:ident, $token_y_address:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();
        create_fee_tier!($client, $dex, $dex_address, fee_tier, alice);

        let init_tick = 0;
        create_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            init_tick,
            alice
        );
    }};
}

#[macro_export]
macro_rules! init_basic_position {
    ($client:ident, $dex:ty, $token:ty, $dex_address:ident, $token_x_address:ident, $token_y_address:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();

        let mint_amount = 10u128.pow(10);
        approve!(
            $client,
            $token,
            $token_x_address,
            $dex_address,
            mint_amount,
            alice
        );
        approve!(
            $client,
            $token,
            $token_y_address,
            $dex_address,
            mint_amount,
            alice
        );

        let pool_key = PoolKey::new($token_x_address, $token_y_address, fee_tier).unwrap();
        let lower_tick = -20;
        let upper_tick = 10;
        let liquidity = Liquidity::from_integer(1000000);

        // liquidityDelta = { v: new BN(1000000).mul(LIQUIDITY_DENOMINATOR) }
        // L_denominator = 10^6

        let pool_before = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            alice
        )
        .unwrap();
        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;
        create_position!(
            $client,
            $dex,
            $dex_address,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        );

        let pool_after = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            alice
        )
        .unwrap();

        assert_eq!(pool_after.liquidity, liquidity);
    }};
}

#[macro_export]
macro_rules! init_cross_position {
    ($client:ident, $dex:ty, $token:ty, $dex_address:ident, $token_x_address:ident, $token_y_address:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();

        let mint_amount = 10u128.pow(10);
        approve!(
            $client,
            $token,
            $token_x_address,
            $dex_address,
            mint_amount,
            alice
        );
        approve!(
            $client,
            $token,
            $token_y_address,
            $dex_address,
            mint_amount,
            alice
        );

        let pool_key = PoolKey::new($token_x_address, $token_y_address, fee_tier).unwrap();
        let lower_tick = -40;
        let upper_tick = -10;
        let liquidity = Liquidity::from_integer(1000000);

        // liquidityDelta = { v: new BN(1000000).mul(LIQUIDITY_DENOMINATOR) }
        // L_denominator = 10^6

        let pool_before = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            alice
        )
        .unwrap();
        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;
        create_position!(
            $client,
            $dex,
            $dex_address,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        );

        let pool_after = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            alice
        )
        .unwrap();

        assert_eq!(pool_after.liquidity, pool_before.liquidity);
    }};
}

#[macro_export]
macro_rules! init_basic_swap {
    ($client:ident, $dex:ty, $token:ty, $dex_address:ident, $token_x_address:ident, $token_y_address:ident) => {{
        let fee = Percentage::from_scale(6, 3);
        let tick_spacing = 10;
        let fee_tier = FeeTier { fee, tick_spacing };
        let pool_key = PoolKey::new($token_x_address, $token_y_address, fee_tier).unwrap();
        let lower_tick = -20;

        let amount = 1000;
        let bob = ink_e2e::bob();
        mint!($client, $token, $token_x_address, address_of!(Bob), amount);
        let amount_x = balance_of!($client, $token, $token_x_address, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!($client, $token, $token_x_address, $dex_address, amount, bob);

        let amount_x = balance_of!($client, $token, $token_x_address, $dex_address);
        let amount_y = balance_of!($client, $token, $token_y_address, $dex_address);
        assert_eq!(amount_x, 500);
        assert_eq!(amount_y, 1000);

        let pool_before = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            pool_key.fee_tier,
            bob
        )
        .unwrap();

        let swap_amount = TokenAmount::new(amount);
        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        swap!(
            $client,
            $dex,
            $dex_address,
            pool_key,
            true,
            swap_amount,
            true,
            slippage,
            bob
        );

        let pool_after = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            bob
        )
        .unwrap();
        assert_eq!(pool_after.liquidity, pool_before.liquidity);
        assert_eq!(pool_after.current_tick_index, lower_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = balance_of!($client, $token, $token_x_address, address_of!(Bob));
        let amount_y = balance_of!($client, $token, $token_y_address, address_of!(Bob));
        assert_eq!(amount_x, 0);
        assert_eq!(amount_y, 993);

        let amount_x = balance_of!($client, $token, $token_x_address, $dex_address);
        let amount_y = balance_of!($client, $token, $token_y_address, $dex_address);
        assert_eq!(amount_x, 1500);
        assert_eq!(amount_y, 7);

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(50000000000000000000000)
        );
        assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0));

        assert_eq!(pool_after.fee_protocol_token_x, TokenAmount::new(1));
        assert_eq!(pool_after.fee_protocol_token_y, TokenAmount::new(0));
    }};
}

#[macro_export]
macro_rules! init_cross_swap {
    ($client:ident, $dex:ty, $token:ty, $dex_address:ident, $token_x_address:ident, $token_y_address:ident) => {{
        let fee = Percentage::from_scale(6, 3);
        let tick_spacing = 10;
        let fee_tier = FeeTier { fee, tick_spacing };
        let pool_key = PoolKey::new($token_x_address, $token_y_address, fee_tier).unwrap();
        let lower_tick = -20;

        let amount = 1000;
        let bob = ink_e2e::bob();
        mint!($client, $token, $token_x_address, address_of!(Bob), amount);
        let amount_x = balance_of!($client, $token, $token_x_address, address_of!(Bob));
        assert_eq!(amount_x, amount);
        approve!($client, $token, $token_x_address, $dex_address, amount, bob);

        let amount_x = balance_of!($client, $token, $token_x_address, $dex_address);
        let amount_y = balance_of!($client, $token, $token_y_address, $dex_address);
        assert_eq!(amount_x, 500);
        assert_eq!(amount_y, 2499);

        let pool_before = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            bob
        )
        .unwrap();

        let swap_amount = TokenAmount::new(amount);
        let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
        swap!(
            $client,
            $dex,
            $dex_address,
            pool_key,
            true,
            swap_amount,
            true,
            slippage,
            bob
        );

        let pool_after = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            bob
        )
        .unwrap();
        let position_liquidity = Liquidity::from_integer(1000000);
        assert_eq!(
            pool_after.liquidity - position_liquidity,
            pool_before.liquidity
        );
        assert_eq!(pool_after.current_tick_index, lower_tick);
        assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

        let amount_x = balance_of!($client, $token, $token_x_address, address_of!(Bob));
        let amount_y = balance_of!($client, $token, $token_y_address, address_of!(Bob));
        assert_eq!(amount_x, 0);
        assert_eq!(amount_y, 990);

        let amount_x = balance_of!($client, $token, $token_x_address, $dex_address);
        let amount_y = balance_of!($client, $token, $token_y_address, $dex_address);
        assert_eq!(amount_x, 1500);
        assert_eq!(amount_y, 1509);

        assert_eq!(
            pool_after.fee_growth_global_x,
            FeeGrowth::new(40000000000000000000000)
        );
        assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0));

        assert_eq!(pool_after.fee_protocol_token_x, TokenAmount::new(2));
        assert_eq!(pool_after.fee_protocol_token_y, TokenAmount::new(0));
    }};
}

#[macro_export]
macro_rules! create_slippage_pool_with_liquidity {
    ($client:ident, $dex:ty, $token:ty, $dex_address:ident, $token_x_address:ident, $token_y_address:ident) => {{
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();
        create_fee_tier!($client, $dex, $dex_address, fee_tier, alice);

        let init_tick = 0;
        create_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            init_tick,
            alice
        );
        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 10,
        };
        let alice = ink_e2e::alice();

        let mint_amount = 10u128.pow(10);
        approve!(
            $client,
            $token,
            $token_x_address,
            $dex_address,
            mint_amount,
            alice
        );
        approve!(
            $client,
            $token,
            $token_y_address,
            $dex_address,
            mint_amount,
            alice
        );

        let pool_key = PoolKey::new($token_x_address, $token_y_address, fee_tier).unwrap();
        let lower_tick = -1000;
        let upper_tick = 1000;
        let liquidity = Liquidity::new(10u128.pow(16));

        let pool_before = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            alice
        )
        .unwrap();
        let slippage_limit_lower = pool_before.sqrt_price;
        let slippage_limit_upper = pool_before.sqrt_price;
        create_position!(
            $client,
            $dex,
            $dex_address,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        );

        let pool_after = get_pool!(
            $client,
            $dex,
            $dex_address,
            $token_x_address,
            $token_y_address,
            fee_tier,
            alice
        )
        .unwrap();

        assert_eq!(pool_after.liquidity, liquidity);

        pool_key
    }};
}

#[macro_export]
macro_rules! init_slippage_dex_and_tokens {
    ($client:ident, $dex:ty, $token:ty) => {{
        let mint_amount = 10u128.pow(23);
        let (token_x, token_y) = create_tokens!($client, $token, $token, mint_amount, mint_amount);

        let protocol_fee = Percentage::from_scale(1, 2);
        let dex = create_dex!($client, $dex, protocol_fee);
        (dex, token_x, token_y)
    }};
}

#[macro_export]
macro_rules! swap_exact_limit {
    ($client:ident, $dex:ty, $dex_address:ident, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $caller:ident) => {{
        let sqrt_price_limit = if $x_to_y {
            SqrtPrice::new(MIN_SQRT_PRICE)
        } else {
            SqrtPrice::new(MAX_SQRT_PRICE)
        };

        let quote_result = quote!(
            $client,
            $dex,
            $dex_address,
            $pool_key,
            $x_to_y,
            $amount,
            $by_amount_in,
            sqrt_price_limit,
            $caller
        )
        .unwrap();
        swap!(
            $client,
            $dex,
            $dex_address,
            $pool_key,
            $x_to_y,
            $amount,
            $by_amount_in,
            quote_result.target_sqrt_price,
            $caller
        );
    }};
}

#[macro_export]
macro_rules! init_dex_and_tokens_max_mint_amount {
    ($client:ident, $dex:ty, $token:ty) => {{
        let mint_amount = u128::MAX;
        let (token_x, token_y) = create_tokens!($client, $token, $token, mint_amount, mint_amount);

        let protocol_fee = Percentage::from_scale(1, 2);
        let dex = create_dex!($client, $dex, protocol_fee);
        (dex, token_x, token_y)
    }};
}

#[macro_export]
macro_rules! mint_with_aprove_for_bob {
    ($client:ident, $token:ty, $token_address:ident, $dex_address:ident, $mint_amount:expr) => {{
        let bob = ink_e2e::bob();
        mint!(
            $client,
            $token,
            $token_address,
            address_of!(Bob),
            $mint_amount
        );
        let amount = balance_of!($client, $token, $token_address, address_of!(Bob));
        assert_eq!(amount, $mint_amount);
        approve!(
            $client,
            $token,
            $token_address,
            $dex_address,
            $mint_amount,
            bob
        );
    }};
}

#[macro_export]
macro_rules! big_deposit_and_swap {
    ($client:ident, $dex:ty, $token:ty, $x_to_y:expr) => {{
        let (dex, token_x, token_y) = init_dex_and_tokens_max_mint_amount!($client, $dex, $token);

        let mint_amount = 2u128.pow(75) - 1;
        let alice = ink_e2e::alice();
        approve!($client, $token, token_x, dex, u128::MAX, alice);
        approve!($client, $token, token_y, dex, u128::MAX, alice);

        let fee_tier = FeeTier {
            fee: Percentage::from_scale(6, 3),
            tick_spacing: 1,
        };
        create_fee_tier!($client, $dex, dex, fee_tier, alice);

        let init_tick = 0;
        create_pool!($client, $dex, dex, token_x, token_y, fee_tier, init_tick, alice);

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
        let pool = get_pool!($client, $dex, dex, token_x, token_y, fee_tier, alice).unwrap();

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

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;
        create_position!(
            $client,
            $dex,
            dex,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        );

        let amount_x = balance_of!($client, $token, token_x, address_of!(Alice));
        let amount_y = balance_of!($client, $token, token_y, address_of!(Alice));
        if $x_to_y {
            assert_eq!(amount_x, 340282366920938463463374607431768211455);
            assert_eq!(amount_y, 340282366920938425684442744474606501888);
        } else {
            assert_eq!(amount_x, 340282366920938425684442744474606501888);
            assert_eq!(amount_y, 340282366920938463463374607431768211455);
        }

        let sqrt_price_limit = if $x_to_y {
            SqrtPrice::new(MIN_SQRT_PRICE)
        } else {
            SqrtPrice::new(MAX_SQRT_PRICE)
        };

        swap!(
            $client,
            $dex,
            dex,
            pool_key,
            $x_to_y,
            TokenAmount(mint_amount),
            true,
            sqrt_price_limit,
            alice
        );

        let amount_x = balance_of!($client, $token, token_x, address_of!(Alice));
        let amount_y = balance_of!($client, $token, token_y, address_of!(Alice));
        if $x_to_y {
            assert_eq!(amount_x, 340282366920938425684442744474606501888);
            assert_ne!(amount_y, 0);
        } else {
            assert_ne!(amount_x, 0);
            assert_eq!(amount_y, 340282366920938425684442744474606501888);
        }
    }};
}

#[macro_export]
macro_rules! multiple_swap {
    ($client:ident, $dex:ty, $token:ty, $x_to_y:expr) => {{
        let (dex, token_x, token_y) = init_dex_and_tokens!($client, $dex, $token);

        let fee_tier = FeeTier {
            fee: Percentage::from_scale(1, 3),
            tick_spacing: 1,
        };
        let alice = ink_e2e::alice();
        create_fee_tier!($client, $dex, dex, fee_tier, alice);

        let init_tick = 0;
        create_pool!($client, $dex, dex, token_x, token_y, fee_tier, init_tick, alice);

        let mint_amount = 10u128.pow(10);
        approve!($client, $token, token_x, dex, mint_amount, alice);
        approve!($client, $token, token_y, dex, mint_amount, alice);

        let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
        let mut upper_tick = 953;
        let mut lower_tick = -upper_tick;

        let amount = 100;
        let pool_data = get_pool!($client, $dex, dex, token_x, token_y, fee_tier, alice).unwrap();
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
            $dex,
            dex,
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            alice
        );

        let bob = ink_e2e::bob();
        if $x_to_y {
            mint!($client, $token, token_x, address_of!(Bob), amount);
            let amount_x = balance_of!($client, $token, token_x, address_of!(Bob));
            assert_eq!(amount_x, amount);
            approve!($client, $token, token_x, dex, amount, bob);
        } else {
            mint!($client, $token, token_y, address_of!(Bob), amount);
            let amount_y = balance_of!($client, $token, token_y, address_of!(Bob));
            assert_eq!(amount_y, amount);
            approve!($client, $token, token_y, dex, amount, bob);
        }

        let swap_amount = TokenAmount(10);
        for i in 1..=10 {
            swap_exact_limit!(
                $client,
                $dex,
                dex,
                pool_key,
                $x_to_y,
                swap_amount,
                true,
                bob
            );
        }

        let pool = get_pool!($client, $dex, dex, token_x, token_y, fee_tier, alice).unwrap();
        if $x_to_y {
            assert_eq!(pool.current_tick_index, -821);
        } else {
            assert_eq!(pool.current_tick_index, 820);
        }
        assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(0));
        assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
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

        let dex_amount_x = balance_of!($client, $token, token_x, dex);
        let dex_amount_y = balance_of!($client, $token, token_y, dex);
        if $x_to_y {
            assert_eq!(dex_amount_x, 200);
            assert_eq!(dex_amount_y, 20);
        } else {
            assert_eq!(dex_amount_x, 20);
            assert_eq!(dex_amount_y, 200);
        }

        let user_amount_x = balance_of!($client, $token, token_x, address_of!(Bob));
        let user_amount_y = balance_of!($client, $token, token_y, address_of!(Bob));
        if $x_to_y {
            assert_eq!(user_amount_x, 0);
            assert_eq!(user_amount_y, 80);
        } else {
            assert_eq!(user_amount_x, 80);
            assert_eq!(user_amount_y, 0);
        }
    }};
}
