#[macro_export]
macro_rules! get_tickmap {
    ($client:ident, $dex:ident, $pool_key:expr, $lower_tick_index:expr, $upper_tick_index:expr , $x_to_y:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call =
            call_builder.get_tickmap($pool_key, $lower_tick_index, $upper_tick_index, $x_to_y);

        $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_protocol_fee {
    ($client:ident, $dex:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_protocol_fee();
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! withdraw_protocol_fee {
    ($client:ident, $dex:ident, $pool_key:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.withdraw_protocol_fee($pool_key);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! change_protocol_fee {
    ($client:ident, $dex:ident, $protocol_fee:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.change_protocol_fee($protocol_fee);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! change_fee_receiver {
    ($client:ident, $dex:ident, $pool_key:expr, $fee_receiver:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.change_fee_receiver($pool_key, $fee_receiver);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(u64::MAX)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! create_position {
    ($client:ident, $dex:ident, $pool_key:expr, $lower_tick:expr, $upper_tick:expr, $liquidity_delta:expr, $slippage_limit_lower:expr, $slippage_limit_upper:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.create_position(
            $pool_key,
            $lower_tick,
            $upper_tick,
            $liquidity_delta,
            $slippage_limit_lower,
            $slippage_limit_upper,
        );
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! swap {
    ($client:ident, $dex:ident, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $sqrt_price_limit:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.swap(
            $pool_key,
            $x_to_y,
            $amount,
            $by_amount_in,
            $sqrt_price_limit,
        );
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};

    ($client:ident, $dex:ident, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $sqrt_price_limit:expr, $caller:ident, $expected_panic: expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.swap(
            $pool_key,
            $x_to_y,
            $amount,
            $by_amount_in,
            $sqrt_price_limit,
        );
        let result = $client.call(&$caller, &call).dry_run().await;

        if result.is_err() {
            assert!(result.unwrap_err().to_string().contains($expected_panic));
        } else {
            std::panic!("Swap did not panic");
        }
    }};
}

#[macro_export]
macro_rules! swap_route {
    ($client:ident, $dex:ident, $amount_in:expr, $expected_amount_out:expr, $slippage:expr, $swaps:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.swap_route($amount_in, $expected_amount_out, $slippage, $swaps);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! quote {
    ($client:ident, $dex:ident, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $sqrt_price_limit:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.quote(
            $pool_key,
            $x_to_y,
            $amount,
            $by_amount_in,
            $sqrt_price_limit,
        );
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! quote_route {
    ($client:ident, $dex:ident, $amount_in:expr, $swaps:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.quote_route($amount_in, $swaps);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! transfer_position {
    ($client:ident, $dex:ident, $index:expr, $receiver:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.transfer_position($index, $receiver);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! get_position {
    ($client:ident, $dex:ident, $index:expr, $caller:ident) => {{
        let owner = AccountId::from($caller.public_key().0);
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_position(owner, $index);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_all_positions {
    ($client:ident, $dex:ident, $caller:ident) => {{
        let owner = AccountId::from($caller.public_key().0);
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_all_positions(owner);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_positions {
    ($client:ident, $dex:ident, $size:expr, $offset:expr, $caller:ident) => {{
        let owner = AccountId::from($caller.public_key().0);
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_positions(owner, $size, $offset);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! update_position_seconds_per_liquidity {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $pool_key:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.update_position_seconds_per_liquidity($index, $pool_key));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.update_position_seconds_per_liquidity($index, $pool_key));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("update_position_seconds_per_liquidity failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! claim_fee {
    ($client:ident, $dex:ident, $index:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.claim_fee($index);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! remove_position {
    ($client:ident, $dex:ident, $index:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.remove_position($index);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! add_fee_tier {
    ($client:ident, $dex:ident, $fee_tier:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.add_fee_tier($fee_tier);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! fee_tier_exist {
    ($client:ident, $dex:ident, $fee_tier:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.fee_tier_exist($fee_tier);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! remove_fee_tier {
    ($client:ident, $dex:ident, $fee_tier:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.remove_fee_tier($fee_tier);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! create_pool {
    ($client:ident, $dex:ident, $token_0:expr, $token_1:expr, $fee_tier:expr, $init_sqrt_price:expr, $init_tick:expr, $caller:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call =
            call_builder.create_pool($token_0, $token_1, $fee_tier, $init_sqrt_price, $init_tick);
        let result = $client
            .call(&$caller, &call)
            .extra_gas_portion(1000)
            .dry_run()
            .await
            .unwrap()
            .return_value();

        if result.is_ok() {
            $client
                .call(&$caller, &call)
                .extra_gas_portion(1000)
                .submit()
                .await
                .unwrap()
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! get_pool {
    ($client:ident, $dex:ident, $token_0:expr, $token_1:expr, $fee_tier:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_pool($token_0, $token_1, $fee_tier);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_tick {
    ($client:ident, $dex:ident, $key:expr, $index:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_tick($key, $index);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! is_tick_initialized {
    ($client:ident, $dex:ident, $key:expr, $index:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.is_tick_initialized($key, $index);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_pools {
    ($client:ident, $dex:ident, $size:expr, $offset:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_pools($size, $offset);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_fee_tiers {
    ($client:ident, $dex:ident) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_fee_tiers();
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_position_ticks {
    ($client:ident, $dex:ident, $owner:expr, $offset:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_position_ticks($owner, $offset);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_liquidity_ticks {
    ($client:ident, $dex:ident, $pool_key:expr, $offset:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_liquidity_ticks($pool_key, $offset);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_liquidity_ticks_amount {
    ($client:ident, $dex:ident, $pool_key:expr, $lower_tick:expr, $upper_tick:expr) => {{
        let mut call_builder = $dex.call_builder::<Invariant>();
        let call = call_builder.get_liquidity_ticks_amount($pool_key, $lower_tick, $upper_tick);
        $client
            .call(&ink_e2e::alice(), &call)
            .dry_run()
            .await
            .unwrap()
            .return_value()
    }};
}
