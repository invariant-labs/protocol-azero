#[macro_export]
macro_rules! get_tickmap {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $current_tick_index:expr, $offset:expr, $amount:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.get_tickmap($pool_key, $current_tick_index, $offset, $amount)
        });
        $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_initialized_chunks {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_initialized_chunks($pool_key));
        $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_protocol_fee {
    ($client:ident, $dex:ty, $dex_address:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_protocol_fee());
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! withdraw_protocol_fee {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.withdraw_protocol_fee($pool_key));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.withdraw_protocol_fee($pool_key));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("withdraw_protocol_fee failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! change_protocol_fee {
    ($client:ident, $dex:ty, $dex_address:expr, $protocol_fee:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.change_protocol_fee($protocol_fee));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.change_protocol_fee($protocol_fee));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("change_protocol_fee failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! change_fee_receiver {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $fee_receiver:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.change_fee_receiver($pool_key, $fee_receiver));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.change_fee_receiver($pool_key, $fee_receiver));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("change_fee_receiver failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! create_position {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $lower_tick:expr, $upper_tick:expr, $liquidity_delta:expr, $slippage_limit_lower:expr, $slippage_limit_upper:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.create_position(
                $pool_key,
                $lower_tick,
                $upper_tick,
                $liquidity_delta,
                $slippage_limit_lower,
                $slippage_limit_upper,
            )
        });
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
                contract.create_position(
                    $pool_key,
                    $lower_tick,
                    $upper_tick,
                    $liquidity_delta,
                    $slippage_limit_lower,
                    $slippage_limit_upper,
                )
            });
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("create_position failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! swap {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $sqrt_price_limit:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.swap(
                $pool_key,
                $x_to_y,
                $amount,
                $by_amount_in,
                $sqrt_price_limit,
            )
        });
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
                contract.swap(
                    $pool_key,
                    $x_to_y,
                    $amount,
                    $by_amount_in,
                    $sqrt_price_limit,
                )
            });
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("swap failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! swap_route {
    ($client:ident, $dex:ty, $dex_address:expr, $amount_in:expr, $expected_amount_out:expr, $slippage:expr, $swaps:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.swap_route($amount_in, $expected_amount_out, $slippage, $swaps)
        });
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
                contract.swap_route($amount_in, $expected_amount_out, $slippage, $swaps)
            });
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("swap_route failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! quote {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $x_to_y:expr, $amount:expr, $by_amount_in:expr, $sqrt_price_limit:expr) => {{
        let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.quote(
                $pool_key,
                $x_to_y,
                $amount,
                $by_amount_in,
                $sqrt_price_limit,
            )
        });
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! quote_route {
    ($client:ident, $dex:ty, $dex_address:expr, $amount_in:expr, $swaps:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.quote_route($amount_in, $swaps));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! transfer_position {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $receiver:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.transfer_position($index, $receiver));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.transfer_position($index, $receiver));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("transfer_position failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! get_position {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $caller:ident) => {{
        let owner = AccountId::from($caller.public_key().0);
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_position(owner, $index));
        $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_all_positions {
    ($client:ident, $dex:ty, $dex_address:expr, $caller:ident) => {{
        let owner = AccountId::from($caller.public_key().0);
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_all_positions(owner));
        $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
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
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $caller:ident) => {{
        let message =
            build_message::<$dex>($dex_address.clone()).call(|contract| contract.claim_fee($index));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.claim_fee($index));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("claim_fee failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! remove_position {
    ($client:ident, $dex:ty, $dex_address:expr, $index:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.remove_position($index));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.remove_position($index));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("remove_position failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! add_fee_tier {
    ($client:ident, $dex:ty, $dex_address:expr, $fee_tier:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.add_fee_tier($fee_tier));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.add_fee_tier($fee_tier));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("add_fee_tier failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! fee_tier_exist {
    ($client:ident, $dex:ty, $dex_address:expr, $fee_tier:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.fee_tier_exist($fee_tier));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! remove_fee_tier {
    ($client:ident, $dex:ty, $dex_address:expr, $fee_tier:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.remove_fee_tier($fee_tier));
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone())
                .call(|contract| contract.remove_fee_tier($fee_tier));
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("remove_fee_tier failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! create_pool {
    ($client:ident, $dex:ty, $dex_address:expr, $token_0:expr, $token_1:expr, $fee_tier:expr, $init_sqrt_price:expr, $init_tick:expr, $caller:ident) => {{
        let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
            contract.create_pool($token_0, $token_1, $fee_tier, $init_sqrt_price, $init_tick)
        });
        let result = $client
            .call_dry_run(&$caller, &message, 0, None)
            .await
            .return_value();

        if result.is_ok() {
            let message = build_message::<$dex>($dex_address.clone()).call(|contract| {
                contract.create_pool($token_0, $token_1, $fee_tier, $init_sqrt_price, $init_tick)
            });
            $client
                .call(&$caller, message, 0, None)
                .await
                .expect("create_pool failed")
                .return_value()
        } else {
            result
        }
    }};
}

#[macro_export]
macro_rules! get_pool {
    ($client:ident, $dex:ty, $dex_address:expr, $token_0:expr, $token_1:expr, $fee_tier:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_pool($token_0, $token_1, $fee_tier));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_tick {
    ($client:ident, $dex:ty, $dex_address:expr, $key:expr, $index:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_tick($key, $index));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! is_tick_initialized {
    ($client:ident, $dex:ty, $dex_address:expr, $key:expr, $index:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.is_tick_initialized($key, $index));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_pools {
    ($client:ident, $dex:ty, $dex_address:expr, $size:expr, $offset:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_pools($size, $offset));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
            .unwrap()
    }};
}

#[macro_export]
macro_rules! get_fee_tiers {
    ($client:ident, $dex:ty, $dex_address:expr) => {{
        let message =
            build_message::<$dex>($dex_address.clone()).call(|contract| contract.get_fee_tiers());
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_position_ticks {
    ($client:ident, $dex:ty, $dex_address:expr, $owner:expr, $offset:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_position_ticks($owner, $offset));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_liquidity_ticks {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr, $offset:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_liquidity_ticks($pool_key, $offset));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}

#[macro_export]
macro_rules! get_liquidity_ticks_amount {
    ($client:ident, $dex:ty, $dex_address:expr, $pool_key:expr) => {{
        let message = build_message::<$dex>($dex_address.clone())
            .call(|contract| contract.get_liquidity_ticks_amount($pool_key));
        $client
            .call_dry_run(&ink_e2e::alice(), &message, 0, None)
            .await
            .return_value()
    }};
}
