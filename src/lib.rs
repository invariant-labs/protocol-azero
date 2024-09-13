#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
mod contracts;
#[cfg(all(test, feature = "e2e-tests"))]
pub mod e2e;
pub mod math;

#[ink::contract]
pub mod invariant {
    use crate::contracts::{
        tick_to_position, CalculateSwapResult, ChangeLiquidityEvent, CreatePositionEvent,
        CrossTickEvent, FeeTier, FeeTiers, InvariantConfig, InvariantTrait, LiquidityTick, Pool,
        PoolKey, PoolKeys, Pools, Position, Positions, QuoteResult, RemovePositionEvent, SwapEvent,
        SwapHop, Tick, Tickmap, Ticks, UpdatePoolTick, CHUNK_SIZE, LIQUIDITY_TICK_LIMIT,
        MAX_TICKMAP_QUERY_SIZE,
    };
    use crate::math::calculate_min_amount_out;
    use crate::math::check_tick;
    use crate::math::percentage::Percentage;
    use crate::math::sqrt_price::SqrtPrice;
    use crate::math::sqrt_price::{get_max_tick, get_min_tick};
    use crate::math::token_amount::TokenAmount;
    use crate::math::types::liquidity::Liquidity;

    use crate::contracts::InvariantError;
    use crate::math::{compute_swap_step, MAX_SQRT_PRICE, MIN_SQRT_PRICE};
    use crate::{balance_of_v1, transfer_from_v1, transfer_v1, withdraw_v1};
    use decimal::*;

    use ink::codegen::TraitCallBuilder;
    use ink::contract_ref;
    use ink::env::DefaultEnvironment;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;
    use token::{PSP22Error, PSP22};
    use traceable_result::{unwrap, TrackableError};

    type PSP22Wrapper = contract_ref!(PSP22);
    type WrappedAZEROWrapper = contract_ref!(WrappedAZERO);

    #[ink::trait_definition]
    pub trait WrappedAZERO {
        #[ink(message, payable)]
        fn deposit(&mut self) -> Result<(), PSP22Error>;
        #[ink(message)]
        fn withdraw(&mut self, value: u128) -> Result<(), PSP22Error>;
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Invariant {
        positions: Positions,
        pools: Pools,
        tickmap: Tickmap,
        ticks: Ticks,
        fee_tiers: FeeTiers,
        pool_keys: PoolKeys,
        config: InvariantConfig,
    }

    impl Invariant {
        #[ink(constructor)]
        pub fn new(protocol_fee: Percentage) -> Self {
            Self {
                config: InvariantConfig {
                    admin: Self::env().caller(),
                    protocol_fee,
                },
                ..Self::default()
            }
        }

        fn create_tick(&mut self, pool_key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
            let current_timestamp = self.env().block_timestamp();

            check_tick(index, pool_key.fee_tier.tick_spacing)
                .map_err(|_| InvariantError::InvalidTickIndexOrTickSpacing)?;

            let pool = self.pools.get(pool_key)?;

            let tick = Tick::create(index, &pool, current_timestamp);
            self.ticks.add(pool_key, index, &tick)?;

            self.tickmap
                .flip(true, index, pool_key.fee_tier.tick_spacing, pool_key);

            Ok(tick)
        }

        fn calculate_swap(
            &self,
            pool_key: PoolKey,
            x_to_y: bool,
            amount: TokenAmount,
            by_amount_in: bool,
            sqrt_price_limit: SqrtPrice,
        ) -> Result<CalculateSwapResult, InvariantError> {
            let current_timestamp = self.env().block_timestamp();

            if amount.is_zero() {
                return Err(InvariantError::AmountIsZero);
            }

            let mut ticks: Vec<Tick> = vec![];

            let mut pool = self.pools.get(pool_key)?;

            if x_to_y {
                if pool.sqrt_price <= sqrt_price_limit
                    || sqrt_price_limit > SqrtPrice::new(MAX_SQRT_PRICE)
                {
                    return Err(InvariantError::WrongLimit);
                }
            } else if pool.sqrt_price >= sqrt_price_limit
                || sqrt_price_limit < SqrtPrice::new(MIN_SQRT_PRICE)
            {
                return Err(InvariantError::WrongLimit);
            }

            let tick_limit = if x_to_y {
                get_min_tick(pool_key.fee_tier.tick_spacing)
            } else {
                get_max_tick(pool_key.fee_tier.tick_spacing)
            };

            let mut remaining_amount = amount;

            let mut total_amount_in = TokenAmount(0);
            let mut total_amount_out = TokenAmount(0);

            let event_start_sqrt_price = pool.sqrt_price;
            let mut event_fee_amount = TokenAmount(0);

            while !remaining_amount.is_zero() {
                let (swap_limit, limiting_tick) = self.tickmap.get_closer_limit(
                    sqrt_price_limit,
                    x_to_y,
                    pool.current_tick_index,
                    pool_key.fee_tier.tick_spacing,
                    pool_key,
                )?;

                let result = unwrap!(compute_swap_step(
                    pool.sqrt_price,
                    swap_limit,
                    pool.liquidity,
                    remaining_amount,
                    by_amount_in,
                    pool_key.fee_tier.fee,
                ));

                // make remaining amount smaller
                if by_amount_in {
                    let intermediate =
                        result
                            .amount_in
                            .checked_add(result.fee_amount)
                            .map_err(|_| {
                                InvariantError::AddOverflow(
                                    result.amount_in.get(),
                                    result.fee_amount.get(),
                                )
                            })?;
                    remaining_amount =
                        remaining_amount.checked_sub(intermediate).map_err(|_| {
                            InvariantError::SubUnderflow(remaining_amount.get(), intermediate.get())
                        })?;
                } else {
                    remaining_amount =
                        remaining_amount
                            .checked_sub(result.amount_out)
                            .map_err(|_| {
                                InvariantError::SubUnderflow(
                                    remaining_amount.get(),
                                    result.amount_out.get(),
                                )
                            })?;
                }

                unwrap!(pool.add_fee(result.fee_amount, x_to_y, self.config.protocol_fee));
                event_fee_amount =
                    event_fee_amount
                        .checked_add(result.fee_amount)
                        .map_err(|_| {
                            InvariantError::AddOverflow(
                                event_fee_amount.get(),
                                result.fee_amount.get(),
                            )
                        })?;

                pool.sqrt_price = result.next_sqrt_price;

                let intermediate = total_amount_in.checked_add(result.amount_in).map_err(|_| {
                    InvariantError::AddOverflow(total_amount_in.get(), result.amount_in.get())
                })?;
                total_amount_in = intermediate.checked_add(result.fee_amount).map_err(|_| {
                    InvariantError::AddOverflow(intermediate.get(), result.fee_amount.get())
                })?;
                total_amount_out =
                    total_amount_out
                        .checked_add(result.amount_out)
                        .map_err(|_| {
                            InvariantError::AddOverflow(
                                total_amount_out.get(),
                                result.amount_out.get(),
                            )
                        })?;

                // Fail if price would go over swap limit
                if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
                    return Err(InvariantError::PriceLimitReached);
                }

                let mut tick_update = {
                    if let Some((tick_index, is_initialized)) = limiting_tick {
                        if is_initialized {
                            let tick = self.ticks.get(pool_key, tick_index)?;
                            UpdatePoolTick::TickInitialized(tick)
                        } else {
                            UpdatePoolTick::TickUninitialized(tick_index)
                        }
                    } else {
                        UpdatePoolTick::NoTick
                    }
                };

                let (amount_to_add, amount_after_tick_update, has_crossed) = pool.update_tick(
                    result,
                    swap_limit,
                    &mut tick_update,
                    remaining_amount,
                    by_amount_in,
                    x_to_y,
                    current_timestamp,
                    self.config.protocol_fee,
                    pool_key.fee_tier,
                );

                remaining_amount = amount_after_tick_update;
                total_amount_in = total_amount_in.checked_add(amount_to_add).unwrap();

                if let UpdatePoolTick::TickInitialized(tick) = tick_update {
                    if has_crossed {
                        ticks.push(tick)
                    }
                }

                let reached_tick_limit = match x_to_y {
                    true => pool.current_tick_index <= tick_limit,
                    false => pool.current_tick_index >= tick_limit,
                };

                if reached_tick_limit {
                    return Err(InvariantError::TickLimitReached);
                }
            }
            if total_amount_out.get() == 0 {
                return Err(InvariantError::NoGainSwap);
            }

            Ok(CalculateSwapResult {
                amount_in: total_amount_in,
                amount_out: total_amount_out,
                start_sqrt_price: event_start_sqrt_price,
                target_sqrt_price: pool.sqrt_price,
                fee: event_fee_amount,
                pool,
                ticks,
            })
        }

        fn route(
            &mut self,
            is_swap: bool,
            amount_in: TokenAmount,
            swaps: Vec<SwapHop>,
        ) -> Result<TokenAmount, InvariantError> {
            let mut next_swap_amount = amount_in;

            for swap in swaps.iter() {
                let SwapHop { pool_key, x_to_y } = *swap;

                let sqrt_price_limit = if x_to_y {
                    SqrtPrice::new(MIN_SQRT_PRICE)
                } else {
                    SqrtPrice::new(MAX_SQRT_PRICE)
                };

                let result = if is_swap {
                    self.swap(pool_key, x_to_y, next_swap_amount, true, sqrt_price_limit)
                } else {
                    self.calculate_swap(pool_key, x_to_y, next_swap_amount, true, sqrt_price_limit)
                }?;

                next_swap_amount = result.amount_out;
            }

            Ok(next_swap_amount)
        }

        fn remove_tick(&mut self, key: PoolKey, tick: Tick) -> Result<(), InvariantError> {
            if !tick.liquidity_gross.is_zero() {
                return Err(InvariantError::NotEmptyTickDeinitialization);
            }

            self.tickmap
                .flip(false, tick.index, key.fee_tier.tick_spacing, key);
            self.ticks.remove(key, tick.index)?;
            Ok(())
        }

        #[allow(clippy::too_many_arguments)]
        fn emit_swap_event(
            &self,
            address: AccountId,
            pool: PoolKey,
            amount_in: TokenAmount,
            amount_out: TokenAmount,
            fee: TokenAmount,
            start_sqrt_price: SqrtPrice,
            target_sqrt_price: SqrtPrice,
            x_to_y: bool,
        ) {
            let timestamp = self.get_timestamp();
            self.env().emit_event(SwapEvent {
                timestamp,
                address,
                pool,
                amount_in,
                amount_out,
                fee,
                start_sqrt_price,
                target_sqrt_price,
                x_to_y,
            });
        }

        fn emit_create_position_event(
            &self,
            address: AccountId,
            pool: PoolKey,
            liquidity: Liquidity,
            lower_tick: i32,
            upper_tick: i32,
            current_sqrt_price: SqrtPrice,
        ) {
            let timestamp = self.get_timestamp();
            self.env().emit_event(CreatePositionEvent {
                timestamp,
                address,
                pool,
                liquidity,
                lower_tick,
                upper_tick,
                current_sqrt_price,
            });
        }

        #[allow(clippy::too_many_arguments)]
        fn emit_change_liquidity_event(
            &self,
            address: AccountId,
            pool: PoolKey,
            delta_liquidity: Liquidity,
            add_liquidity: bool,
            lower_tick: i32,
            upper_tick: i32,
            current_sqrt_price: SqrtPrice,
        ) {
            let timestamp = self.get_timestamp();
            self.env().emit_event(ChangeLiquidityEvent {
                timestamp,
                address,
                pool,
                delta_liquidity,
                add_liquidity,
                lower_tick,
                upper_tick,
                current_sqrt_price,
            });
        }

        fn emit_remove_position_event(
            &self,
            address: AccountId,
            pool: PoolKey,
            liquidity: Liquidity,
            lower_tick: i32,
            upper_tick: i32,
            current_sqrt_price: SqrtPrice,
        ) {
            let timestamp = self.get_timestamp();
            self.env().emit_event(RemovePositionEvent {
                timestamp,
                address,
                pool,
                liquidity,
                lower_tick,
                upper_tick,
                current_sqrt_price,
            });
        }

        fn emit_cross_tick_event(&self, address: AccountId, pool: PoolKey, indexes: Vec<i32>) {
            let timestamp = self.get_timestamp();
            self.env().emit_event(CrossTickEvent {
                timestamp,
                address,
                pool,
                indexes,
            });
        }

        fn get_timestamp(&self) -> u64 {
            self.env().block_timestamp()
        }

        fn tickmap_slice(
            &self,
            range: impl Iterator<Item = u16>,
            pool_key: PoolKey,
        ) -> Vec<(u16, u64)> {
            let mut tickmap_slice: Vec<(u16, u64)> = vec![];

            for chunk_index in range {
                if let Some(chunk) = self.tickmap.bitmap.get((chunk_index, pool_key)) {
                    tickmap_slice.push((chunk_index, chunk));

                    if tickmap_slice.len() == MAX_TICKMAP_QUERY_SIZE {
                        return tickmap_slice;
                    }
                }
            }

            tickmap_slice
        }
    }

    impl InvariantTrait for Invariant {
        #[ink(message)]
        fn get_protocol_fee(&self) -> Percentage {
            self.config.protocol_fee
        }

        #[ink(message)]
        fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            let mut pool = self.pools.get(pool_key)?;

            if pool.fee_receiver != caller {
                return Err(InvariantError::NotFeeReceiver);
            }

            let (fee_protocol_token_x, fee_protocol_token_y) = pool.withdraw_protocol_fee();
            self.pools.update(pool_key, &pool)?;

            transfer_v1!(
                pool_key.token_x,
                pool.fee_receiver,
                fee_protocol_token_x.get()
            );

            transfer_v1!(
                pool_key.token_y,
                pool.fee_receiver,
                fee_protocol_token_y.get()
            );

            Ok(())
        }

        #[ink(message)]
        fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.config.admin {
                return Err(InvariantError::NotAdmin);
            }

            self.config.protocol_fee = protocol_fee;
            Ok(())
        }

        #[ink(message)]
        fn change_fee_receiver(
            &mut self,
            pool_key: PoolKey,
            fee_receiver: AccountId,
        ) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.config.admin {
                return Err(InvariantError::NotAdmin);
            }

            let mut pool = self.pools.get(pool_key)?;
            pool.fee_receiver = fee_receiver;
            self.pools.update(pool_key, &pool)?;

            Ok(())
        }

        #[ink(message)]
        fn create_position(
            &mut self,
            pool_key: PoolKey,
            lower_tick: i32,
            upper_tick: i32,
            liquidity_delta: Liquidity,
            slippage_limit_lower: SqrtPrice,
            slippage_limit_upper: SqrtPrice,
        ) -> Result<Position, InvariantError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();
            let current_timestamp = self.env().block_timestamp();
            let current_block_number = self.env().block_number() as u64;

            // liquidity delta = 0 => return
            if liquidity_delta == Liquidity::new(0) {
                return Err(InvariantError::ZeroLiquidity);
            }

            if lower_tick == upper_tick {
                return Err(InvariantError::InvalidTickIndex);
            }

            let mut pool = self.pools.get(pool_key)?;

            let mut lower_tick = self
                .ticks
                .get(pool_key, lower_tick)
                .unwrap_or_else(|_| Self::create_tick(self, pool_key, lower_tick).unwrap());

            let mut upper_tick = self
                .ticks
                .get(pool_key, upper_tick)
                .unwrap_or_else(|_| Self::create_tick(self, pool_key, upper_tick).unwrap());

            let (position, x, y) = Position::create(
                &mut pool,
                pool_key,
                &mut lower_tick,
                &mut upper_tick,
                current_timestamp,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                current_block_number,
                pool_key.fee_tier.tick_spacing,
            )?;

            self.pools.update(pool_key, &pool)?;

            self.positions.add(caller, &position);

            self.ticks.update(pool_key, lower_tick.index, &lower_tick)?;
            self.ticks.update(pool_key, upper_tick.index, &upper_tick)?;

            transfer_from_v1!(pool_key.token_x, caller, contract, x.get());
            transfer_from_v1!(pool_key.token_y, caller, contract, y.get());

            self.emit_create_position_event(
                caller,
                pool_key,
                liquidity_delta,
                lower_tick.index,
                upper_tick.index,
                pool.sqrt_price,
            );
            Ok(position)
        }

        #[ink(message)]
        fn change_liquidity(
            &mut self,
            index: u32,
            delta_liquidity: Liquidity,
            add_liquidity: bool,
            slippage_limit_lower: SqrtPrice,
            slippage_limit_upper: SqrtPrice,
        ) -> Result<(), InvariantError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();

            let mut position = self.positions.get(caller, index)?;
            let pool_key = position.pool_key;

            let mut pool = self.pools.get(pool_key)?;
            let mut lower_tick = self.ticks.get(pool_key, position.lower_tick_index)?;
            let mut upper_tick = self.ticks.get(pool_key, position.upper_tick_index)?;

            if !add_liquidity && delta_liquidity == position.liquidity {
                return Err(InvariantError::ZeroLiquidity);
            }

            if delta_liquidity.get() == 0 {
                return Err(InvariantError::LiquidityChangeZero);
            }

            if pool.sqrt_price < slippage_limit_lower || pool.sqrt_price > slippage_limit_upper {
                return Err(InvariantError::PriceLimitReached);
            }

            let (x, y) = unwrap!(position.modify(
                &mut pool,
                &mut upper_tick,
                &mut lower_tick,
                delta_liquidity,
                add_liquidity,
                self.get_timestamp(),
                pool_key.fee_tier.tick_spacing,
            ));

            self.pools.update(pool_key, &pool)?;
            self.positions.update(caller, index, &position)?;
            self.ticks.update(pool_key, lower_tick.index, &lower_tick)?;
            self.ticks.update(pool_key, upper_tick.index, &upper_tick)?;

            let x_is_zero = x.get() == 0;
            let y_is_zero = y.get() == 0;

            if y_is_zero && x_is_zero {
                return Err(InvariantError::AmountIsZero);
            }

            if !x_is_zero {
                if add_liquidity {
                    transfer_from_v1!(pool_key.token_x, caller, contract, x.get());
                } else {
                    transfer_v1!(pool_key.token_x, caller, x.get());
                }
            }

            if !y_is_zero {
                if add_liquidity {
                    transfer_from_v1!(pool_key.token_y, caller, contract, y.get());
                } else {
                    transfer_v1!(pool_key.token_y, caller, y.get());
                }
            }

            self.emit_change_liquidity_event(
                caller,
                pool_key,
                delta_liquidity,
                add_liquidity,
                lower_tick.index,
                upper_tick.index,
                pool.sqrt_price,
            );

            Ok(())
        }

        #[ink(message)]
        fn swap(
            &mut self,
            pool_key: PoolKey,
            x_to_y: bool,
            amount: TokenAmount,
            by_amount_in: bool,
            sqrt_price_limit: SqrtPrice,
        ) -> Result<CalculateSwapResult, InvariantError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();

            let calculate_swap_result =
                self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

            let mut crossed_tick_indexes: Vec<i32> = vec![];

            for tick in calculate_swap_result.ticks.iter() {
                self.ticks.update(pool_key, tick.index, tick)?;
                crossed_tick_indexes.push(tick.index);
            }

            if !crossed_tick_indexes.is_empty() {
                self.emit_cross_tick_event(caller, pool_key, crossed_tick_indexes);
            }

            self.pools.update(pool_key, &calculate_swap_result.pool)?;

            if x_to_y {
                transfer_from_v1!(
                    pool_key.token_x,
                    caller,
                    contract,
                    calculate_swap_result.amount_in.get()
                );
                transfer_v1!(
                    pool_key.token_y,
                    caller,
                    calculate_swap_result.amount_out.get()
                );
            } else {
                transfer_from_v1!(
                    pool_key.token_y,
                    caller,
                    contract,
                    calculate_swap_result.amount_in.get()
                );
                transfer_v1!(
                    pool_key.token_x,
                    caller,
                    calculate_swap_result.amount_out.get()
                );
            };

            self.emit_swap_event(
                caller,
                pool_key,
                calculate_swap_result.amount_in,
                calculate_swap_result.amount_out,
                calculate_swap_result.fee,
                calculate_swap_result.start_sqrt_price,
                calculate_swap_result.target_sqrt_price,
                x_to_y,
            );

            Ok(calculate_swap_result)
        }

        #[ink(message)]
        fn swap_route(
            &mut self,
            amount_in: TokenAmount,
            expected_amount_out: TokenAmount,
            slippage: Percentage,
            swaps: Vec<SwapHop>,
        ) -> Result<(), InvariantError> {
            let amount_out = self.route(true, amount_in, swaps)?;

            let min_amount_out = calculate_min_amount_out(expected_amount_out, slippage);

            if amount_out < min_amount_out {
                return Err(InvariantError::AmountUnderMinimumAmountOut);
            }

            Ok(())
        }

        #[ink(message)]
        fn quote(
            &self,
            pool_key: PoolKey,
            x_to_y: bool,
            amount: TokenAmount,
            by_amount_in: bool,
            sqrt_price_limit: SqrtPrice,
        ) -> Result<QuoteResult, InvariantError> {
            let calculate_swap_result =
                self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

            Ok(QuoteResult {
                amount_in: calculate_swap_result.amount_in,
                amount_out: calculate_swap_result.amount_out,
                target_sqrt_price: calculate_swap_result.pool.sqrt_price,
                ticks: calculate_swap_result.ticks,
            })
        }

        #[ink(message)]
        fn quote_route(
            &mut self,
            amount_in: TokenAmount,
            swaps: Vec<SwapHop>,
        ) -> Result<TokenAmount, InvariantError> {
            let amount_out = self.route(false, amount_in, swaps)?;

            Ok(amount_out)
        }

        #[ink(message)]
        fn transfer_position(
            &mut self,
            index: u32,
            receiver: AccountId,
        ) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            self.positions.transfer(caller, index, receiver)?;

            Ok(())
        }

        #[ink(message)]
        fn get_position(
            &mut self,
            owner_id: AccountId,
            index: u32,
        ) -> Result<Position, InvariantError> {
            self.positions.get(owner_id, index)
        }

        #[ink(message)]
        fn get_positions(
            &mut self,
            owner_id: AccountId,
            size: u32,
            offset: u32,
        ) -> Result<(Vec<(Position, Pool)>, u32), InvariantError> {
            let positions = self.positions.get_all(owner_id, size, offset);
            let mut entries = vec![];

            for position in &positions {
                let pool = self.pools.get(position.pool_key)?;
                entries.push((*position, pool))
            }

            Ok((entries, self.positions.get_length(owner_id)))
        }

        #[ink(message)]
        fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError> {
            let caller = self.env().caller();
            let current_timestamp = self.env().block_timestamp();

            let mut position = self.positions.get(caller, index)?;

            let mut lower_tick = self
                .ticks
                .get(position.pool_key, position.lower_tick_index)?;

            let mut upper_tick = self
                .ticks
                .get(position.pool_key, position.upper_tick_index)?;

            let mut pool = self.pools.get(position.pool_key)?;

            let (x, y) = position.claim_fee(
                &mut pool,
                &mut upper_tick,
                &mut lower_tick,
                current_timestamp,
            );

            self.positions.update(caller, index, &position)?;
            self.pools.update(position.pool_key, &pool)?;
            self.ticks
                .update(position.pool_key, upper_tick.index, &upper_tick)?;
            self.ticks
                .update(position.pool_key, lower_tick.index, &lower_tick)?;

            if x.get() > 0 {
                transfer_v1!(position.pool_key.token_x, caller, x.get());
            }

            if y.get() > 0 {
                transfer_v1!(position.pool_key.token_y, caller, y.get());
            }

            Ok((x, y))
        }

        #[ink(message)]
        fn remove_position(
            &mut self,
            index: u32,
        ) -> Result<(TokenAmount, TokenAmount), InvariantError> {
            let caller = self.env().caller();
            let current_timestamp = self.env().block_timestamp();

            let mut position = self.positions.get(caller, index)?;
            let withdrawed_liquidity = position.liquidity;

            let mut lower_tick = self
                .ticks
                .get(position.pool_key, position.lower_tick_index)?;

            let mut upper_tick = self
                .ticks
                .get(position.pool_key, position.upper_tick_index)?;

            let pool = &mut self.pools.get(position.pool_key)?;

            let (amount_x, amount_y, deinitialize_lower_tick, deinitialize_upper_tick) = position
                .remove(
                    pool,
                    current_timestamp,
                    &mut lower_tick,
                    &mut upper_tick,
                    position.pool_key.fee_tier.tick_spacing,
                );

            self.pools.update(position.pool_key, pool)?;

            if deinitialize_lower_tick {
                self.remove_tick(position.pool_key, lower_tick)?;
            } else {
                self.ticks
                    .update(position.pool_key, position.lower_tick_index, &lower_tick)?;
            }

            if deinitialize_upper_tick {
                self.remove_tick(position.pool_key, upper_tick)?;
            } else {
                self.ticks
                    .update(position.pool_key, position.upper_tick_index, &upper_tick)?;
            }

            self.positions.remove(caller, index)?;

            transfer_v1!(position.pool_key.token_x, caller, amount_x.get());
            transfer_v1!(position.pool_key.token_y, caller, amount_y.get());

            self.emit_remove_position_event(
                caller,
                position.pool_key,
                withdrawed_liquidity,
                lower_tick.index,
                upper_tick.index,
                pool.sqrt_price,
            );
            Ok((amount_x, amount_y))
        }

        #[ink(message)]
        fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if fee_tier.tick_spacing == 0 || fee_tier.tick_spacing > 100 {
                return Err(InvariantError::InvalidTickSpacing);
            }

            if fee_tier.fee >= Percentage::from_integer(1) {
                return Err(InvariantError::InvalidFee);
            }

            if caller != self.config.admin {
                return Err(InvariantError::NotAdmin);
            }

            self.fee_tiers.add(fee_tier)?;

            Ok(())
        }

        #[ink(message)]
        fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.config.admin {
                return Err(InvariantError::NotAdmin);
            }

            self.fee_tiers.remove(fee_tier)?;

            Ok(())
        }

        #[ink(message)]
        fn fee_tier_exist(&self, fee_tier: FeeTier) -> bool {
            self.fee_tiers.contains(fee_tier)
        }

        // Pools
        #[ink(message)]
        fn create_pool(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            fee_tier: FeeTier,
            init_sqrt_price: SqrtPrice,
            init_tick: i32,
        ) -> Result<(), InvariantError> {
            let current_timestamp = self.env().block_timestamp();

            if !self.fee_tiers.contains(fee_tier) {
                return Err(InvariantError::FeeTierNotFound);
            };

            check_tick(init_tick, fee_tier.tick_spacing)
                .map_err(|_| InvariantError::InvalidInitTick)?;

            let pool_key = PoolKey::new(token_0, token_1, fee_tier)?;
            if self.pools.get(pool_key).is_ok() {
                return Err(InvariantError::PoolAlreadyExist);
            };
            let pool = Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                fee_tier.tick_spacing,
                self.config.admin,
            )?;
            self.pools.add(pool_key, &pool)?;
            self.pool_keys.add(pool_key)?;

            Ok(())
        }

        #[ink(message)]
        fn get_pool(
            &self,
            token_0: AccountId,
            token_1: AccountId,
            fee_tier: FeeTier,
        ) -> Result<Pool, InvariantError> {
            let key: PoolKey = PoolKey::new(token_0, token_1, fee_tier)?;
            let pool = self.pools.get(key)?;

            Ok(pool)
        }

        #[ink(message)]
        fn get_all_pools_for_pair(
            &self,
            token0: AccountId,
            token1: AccountId,
        ) -> Result<Vec<(FeeTier, Pool)>, InvariantError> {
            let fee_tiers = self.fee_tiers.get_all();
            let mut pools: Vec<(FeeTier, Pool)> = vec![];
            for fee_tier in fee_tiers {
                let pool_key = PoolKey::new(token0, token1, fee_tier)?;
                if let Ok(pool) = self.pools.get(pool_key) {
                    pools.push((fee_tier, pool));
                }
            }
            Ok(pools)
        }

        #[ink(message)]
        fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
            self.ticks.get(key, index)
        }

        #[ink(message)]
        fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool {
            self.tickmap.get(index, key.fee_tier.tick_spacing, key)
        }

        #[ink(message)]
        fn get_pool_keys(
            &self,
            size: u16,
            offset: u16,
        ) -> Result<(Vec<PoolKey>, u16), InvariantError> {
            let pool_keys = self.pool_keys.get_all(size, offset);
            let pool_keys_count = self.pool_keys.count();
            Ok((pool_keys, pool_keys_count))
        }

        #[ink(message)]
        fn get_fee_tiers(&self) -> Vec<FeeTier> {
            self.fee_tiers.get_all()
        }

        #[ink(message)]
        fn get_position_with_associates(
            &self,
            owner: AccountId,
            index: u32,
        ) -> Result<(Position, Pool, Tick, Tick), InvariantError> {
            let position = self.positions.get(owner, index)?;
            let pool = self.pools.get(position.pool_key)?;
            let tick_lower = self
                .ticks
                .get(position.pool_key, position.lower_tick_index)?;
            let tick_upper = self
                .ticks
                .get(position.pool_key, position.upper_tick_index)?;
            Ok((position, pool, tick_lower, tick_upper))
        }

        #[ink(message)]
        fn get_tickmap(
            &self,
            pool_key: PoolKey,
            lower_tick_index: i32,
            upper_tick_index: i32,
            x_to_y: bool,
        ) -> Vec<(u16, u64)> {
            let tick_spacing = pool_key.fee_tier.tick_spacing;
            let (start_chunk, _) = tick_to_position(lower_tick_index, tick_spacing);
            let (end_chunk, _) = tick_to_position(upper_tick_index, tick_spacing);

            if x_to_y {
                self.tickmap_slice((start_chunk..=end_chunk).rev(), pool_key)
            } else {
                self.tickmap_slice(start_chunk..=end_chunk, pool_key)
            }
        }

        #[ink(message)]
        fn get_liquidity_ticks(
            &self,
            pool_key: PoolKey,
            tickmap: Vec<i32>,
        ) -> Result<Vec<LiquidityTick>, InvariantError> {
            let mut liqudity_ticks: Vec<LiquidityTick> = vec![];

            if tickmap.len() > LIQUIDITY_TICK_LIMIT {
                return Err(InvariantError::TickLimitReached);
            }

            for index in tickmap {
                let tick = LiquidityTick::from(self.ticks.get(pool_key, index).unwrap());

                liqudity_ticks.push(tick);
            }

            Ok(liqudity_ticks)
        }

        #[ink(message)]
        fn get_user_position_amount(&self, owner: AccountId) -> u32 {
            self.positions.get_length(owner)
        }

        #[ink(message)]
        fn get_liquidity_ticks_amount(
            &self,
            pool_key: PoolKey,
            lower_tick: i32,
            upper_tick: i32,
        ) -> Result<u32, InvariantError> {
            let tick_spacing = pool_key.fee_tier.tick_spacing;
            if tick_spacing == 0 {
                return Err(InvariantError::InvalidTickSpacing);
            };

            if lower_tick.checked_rem(tick_spacing as i32).unwrap() != 0
                || upper_tick.checked_rem(tick_spacing as i32).unwrap() != 0
            {
                return Err(InvariantError::InvalidTickIndex);
            }

            let max_tick = get_max_tick(tick_spacing);
            let min_tick = get_min_tick(tick_spacing);

            if lower_tick < min_tick || upper_tick > max_tick {
                return Err(InvariantError::InvalidTickIndex);
            };

            let (min_chunk_index, min_bit) = tick_to_position(lower_tick, tick_spacing);
            let (max_chunk_index, max_bit) = tick_to_position(upper_tick, tick_spacing);

            let active_bits_in_range = |chunk: u64, min_bit: u8, max_bit: u8| {
                let range: u64 = (chunk >> min_bit)
                    & (1u64
                        << (max_bit as u32)
                            .checked_sub(min_bit as u32)
                            .unwrap()
                            .checked_add(1)
                            .unwrap())
                    .checked_sub(1)
                    .unwrap();
                range.count_ones()
            };

            let min_chunk = self
                .tickmap
                .bitmap
                .get((min_chunk_index, pool_key))
                .unwrap_or(0);

            if max_chunk_index == min_chunk_index {
                return Ok(active_bits_in_range(min_chunk, min_bit, max_bit));
            }

            let max_chunk = self
                .tickmap
                .bitmap
                .get((max_chunk_index, pool_key))
                .unwrap_or(0);

            let mut amount: u32 = 0;
            amount = amount
                .checked_add(active_bits_in_range(
                    min_chunk,
                    min_bit,
                    (CHUNK_SIZE - 1) as u8,
                ))
                .unwrap();
            amount = amount
                .checked_add(active_bits_in_range(max_chunk, 0, max_bit))
                .unwrap();

            for i in (min_chunk_index.checked_add(1).unwrap())..max_chunk_index {
                let chunk = self.tickmap.bitmap.get((i, pool_key)).unwrap_or(0);

                amount = amount.checked_add(chunk.count_ones()).unwrap();
            }

            Ok(amount)
        }

        #[ink(message)]
        fn withdraw_all_wazero(&self, address: AccountId) -> Result<(), InvariantError> {
            let caller = self.env().caller();
            let contract = self.env().account_id();

            let balance = balance_of_v1!(address, caller);
            if balance > 0 {
                transfer_from_v1!(address, caller, contract, balance);
                withdraw_v1!(address, balance);
                self.env()
                    .transfer(caller, balance)
                    .map_err(|_| InvariantError::TransferError)?;
            }

            Ok(())
        }

        #[ink(message)]
        fn set_code(&mut self, code_hash: Hash) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.config.admin {
                return Err(InvariantError::NotAdmin);
            }

            ink::env::set_code_hash::<DefaultEnvironment>(&code_hash)
                .map_err(|_| InvariantError::SetCodeHashError)?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {

        use super::*;

        use crate::math::consts::MAX_TICK;
        use crate::math::percentage::Percentage;
        use crate::math::sqrt_price::calculate_sqrt_price;

        #[ink::test]
        fn initialize_works() {
            let _ = Invariant::new(Percentage::new(0));
        }

        #[ink::test]
        fn test_add_pool() {
            let mut contract = Invariant::new(Percentage::new(0));
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 1,
            };

            let init_sqrt_price = calculate_sqrt_price(0).unwrap();

            contract.add_fee_tier(fee_tier).unwrap();

            let result = contract.create_pool(
                token_0,
                token_1,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
                init_sqrt_price,
                0,
            );
            assert!(result.is_ok());
            let result = contract.create_pool(
                token_1,
                token_0,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
                init_sqrt_price,
                0,
            );
            assert_eq!(result, Err(InvariantError::PoolAlreadyExist));
        }

        #[ink::test]
        fn test_get_pool() {
            let mut contract = Invariant::new(Percentage::new(0));
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
            let init_sqrt_price = calculate_sqrt_price(0).unwrap();
            let result = contract.get_pool(
                token_1,
                token_0,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
            );
            assert_eq!(result, Err(InvariantError::PoolNotFound));

            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 1,
            };

            contract.add_fee_tier(fee_tier).unwrap();

            let result = contract.create_pool(token_0, token_1, fee_tier, init_sqrt_price, 0);
            assert!(result.is_ok());
            let result = contract.get_pool(
                token_1,
                token_0,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
            );
            assert!(result.is_ok());
        }

        #[ink::test]
        fn create_tick() {
            let mut contract = Invariant::new(Percentage::new(0));
            let init_sqrt_price = calculate_sqrt_price(0).unwrap();
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 2,
            };
            let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
            let result = contract.create_tick(pool_key, MAX_TICK + 1);
            assert_eq!(result, Err(InvariantError::InvalidTickIndexOrTickSpacing));
            let result = contract.create_tick(pool_key, 1);
            assert_eq!(result, Err(InvariantError::InvalidTickIndexOrTickSpacing));
            let result = contract.create_tick(pool_key, 0);
            assert_eq!(result, Err(InvariantError::PoolNotFound));

            contract.add_fee_tier(fee_tier).unwrap();
            let _ = contract.create_pool(
                pool_key.token_x,
                pool_key.token_y,
                pool_key.fee_tier,
                init_sqrt_price,
                0,
            );
            let result = contract.create_tick(pool_key, 0);
            assert!(result.is_ok());
        }

        #[ink::test]
        fn test_fee_tiers() {
            let mut contract = Invariant::new(Percentage::new(0));
            let fee_tier = FeeTier::new(Percentage::new(1), 10u16).unwrap();
            let fee_tier_value = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 10u16,
            };

            contract.add_fee_tier(fee_tier_value).unwrap();
            assert_eq!(contract.fee_tiers.get_all().len(), 1);
            contract.add_fee_tier(fee_tier_value).unwrap_err();
            contract.remove_fee_tier(fee_tier).unwrap();
            assert_eq!(contract.fee_tiers.get_all().len(), 0);
        }
    }
}
