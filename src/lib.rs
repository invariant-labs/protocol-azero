#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
mod contracts;
pub mod math;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum InvariantError {
    UnauthorizedAdmin,
    PoolAlreadyExist,
    PoolNotFound,
    TickAlreadyExist,
    InvalidTickIndexOrTickSpacing,
    PositionNotFound,
    TickNotFound,
    FeeTierNotFound,
    AmountIsZero,
    WrongLimit,
    PriceLimitReached,
    NoGainSwap,
    InvalidTickSpacing,
    FeeTierAlreadyExist,
    UnauthorizedFeeReceiver,
    ZeroLiquidity,
    TransferError,
    TokensAreTheSame,
    AmountUnderMinimumAmountOut,
}
#[ink::contract]
pub mod contract {
    use crate::InvariantError;
    // use math::fee_growth::FeeGrowth;
    use crate::contracts::state::State;
    use crate::contracts::FeeTierKey;
    use crate::contracts::Invariant;
    use crate::contracts::Pool;
    use crate::contracts::Tick;
    use crate::contracts::Tickmap;
    use crate::contracts::{FeeTier, FeeTiers, PoolKey, Pools, Position, Positions, Ticks}; //
    use crate::math::calculate_min_amount_out;
    use crate::math::check_tick;
    use crate::math::percentage::Percentage;
    use crate::math::sqrt_price::sqrt_price::SqrtPrice;
    use crate::math::token_amount::TokenAmount;
    use crate::math::types::liquidity::Liquidity;
    use crate::math::{compute_swap_step, MAX_SQRT_PRICE, MIN_SQRT_PRICE};
    use decimal::*;
    use ink::contract_ref;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;
    use token::PSP22;
    use traceable_result::unwrap;

    #[ink(event)]
    pub struct CreatePositionEvent {
        #[ink(topic)]
        timestamp: u64,
        address: AccountId,
        pool: PoolKey,
        liquidity: Liquidity,
        lower_tick: i32,
        upper_tick: i32,
        current_sqrt_price: SqrtPrice,
    }
    #[ink(event)]
    pub struct CrossTickEvent {
        #[ink(topic)]
        timestamp: u64,
        address: AccountId,
        pool: PoolKey,
        index: i32,
    }

    #[ink(event)]
    pub struct RemovePositionEvent {
        #[ink(topic)]
        timestamp: u64,
        address: AccountId,
        pool: PoolKey,
        liquidity: Liquidity,
        lower_tick: i32,
        upper_tick: i32,
        current_sqrt_price: SqrtPrice,
    }
    #[ink(event)]
    pub struct SwapEvent {
        #[ink(topic)]
        timestamp: u64,
        address: AccountId,
        pool: PoolKey,
        amount_in: TokenAmount,
        amount_out: TokenAmount,
        fee: TokenAmount,
        start_sqrt_price: SqrtPrice,
        target_sqrt_price: SqrtPrice,
        x_to_y: bool,
    }

    #[derive(scale::Decode, Default, scale::Encode, Clone, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,)
    )]
    pub struct CalculateSwapResult {
        pub amount_in: TokenAmount,
        pub amount_out: TokenAmount,
        pub start_sqrt_price: SqrtPrice,
        pub target_sqrt_price: SqrtPrice,
        pub fee: TokenAmount,
        pub pool: Pool,
        pub ticks: Vec<Tick>,
    }

    #[derive(scale::Decode, Default, scale::Encode, Clone, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,)
    )]
    pub struct Hop {
        pool_key: PoolKey,
        x_to_y: bool,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        positions: Positions,
        fee_tiers: FeeTiers,
        pools: Pools,
        tickmap: Tickmap,
        ticks: Ticks,
        fee_tier_keys: Vec<FeeTierKey>,
        pool_keys: Vec<PoolKey>,
        state: State,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(protocol_fee: Percentage) -> Self {
            Self {
                state: State {
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
            let caller = self.env().caller();
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
            } else {
                if pool.sqrt_price >= sqrt_price_limit
                    || sqrt_price_limit < SqrtPrice::new(MIN_SQRT_PRICE)
                {
                    return Err(InvariantError::WrongLimit);
                }
            }

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
                );

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
                    remaining_amount -= result.amount_in + result.fee_amount;
                } else {
                    remaining_amount -= result.amount_out;
                }

                pool.add_fee(result.fee_amount, x_to_y, self.state.protocol_fee);
                event_fee_amount += result.fee_amount;

                pool.sqrt_price = result.next_sqrt_price;

                total_amount_in += result.amount_in + result.fee_amount;
                total_amount_out += result.amount_out;

                // Fail if price would go over swap limit
                if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
                    return Err(InvariantError::PriceLimitReached);
                }

                // TODO: refactor
                let mut tick = Tick::default();

                let update_limiting_tick = limiting_tick.map(|(index, bool)| {
                    if bool {
                        tick = self.ticks.get(pool_key, index).unwrap();
                        (index, Some(&mut tick))
                    } else {
                        (index, None)
                    }
                });

                let has_crossed = pool.cross_tick(
                    result,
                    swap_limit,
                    update_limiting_tick,
                    &mut remaining_amount,
                    by_amount_in,
                    x_to_y,
                    current_timestamp,
                    &mut total_amount_in,
                    self.state.protocol_fee,
                    pool_key.fee_tier,
                );
                if has_crossed {
                    self.emit_cross_tick_event(caller, pool_key, limiting_tick.unwrap().0)
                }

                ticks.push(tick);
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

        fn remove_tick(&mut self, key: PoolKey, index: i32) {
            self.ticks.remove(key, index);
        }

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
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                SwapEvent {
                    timestamp,
                    address,
                    pool,
                    amount_in,
                    amount_out,
                    fee,
                    start_sqrt_price,
                    target_sqrt_price,
                    x_to_y,
                },
            );
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
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                CreatePositionEvent {
                    timestamp,
                    address,
                    pool,
                    liquidity,
                    lower_tick,
                    upper_tick,
                    current_sqrt_price,
                },
            );
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
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                RemovePositionEvent {
                    timestamp,
                    address,
                    pool,
                    liquidity,
                    lower_tick,
                    upper_tick,
                    current_sqrt_price,
                },
            );
        }

        fn emit_cross_tick_event(&self, address: AccountId, pool: PoolKey, index: i32) {
            let timestamp = self.get_timestamp();
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                CrossTickEvent {
                    timestamp,
                    address,
                    pool,
                    index,
                },
            );
        }

        fn get_timestamp(&self) -> u64 {
            self.env().block_timestamp()
        }
    }

    impl Invariant for Contract {
        #[ink(message)]
        fn get_protocol_fee(&self) -> Percentage {
            self.state.protocol_fee
        }

        #[ink(message)]
        fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            let mut pool = self.pools.get(pool_key)?;

            if pool.fee_receiver != caller {
                return Err(InvariantError::UnauthorizedFeeReceiver);
            }

            let (fee_protocol_token_x, fee_protocol_token_y) = pool.withdraw_protocol_fee(pool_key);
            self.pools.update(pool_key, &pool)?;

            let mut token_x: contract_ref!(PSP22) = pool_key.token_x.into();
            token_x
                .transfer(pool.fee_receiver, fee_protocol_token_x.get(), vec![])
                .map_err(|_| InvariantError::TransferError)?;
            let mut token_y: contract_ref!(PSP22) = pool_key.token_y.into();
            token_y
                .transfer(pool.fee_receiver, fee_protocol_token_y.get(), vec![])
                .map_err(|_| InvariantError::TransferError)?;

            Ok(())
        }

        #[ink(message)]
        fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.state.admin {
                return Err(InvariantError::UnauthorizedAdmin);
            }

            self.state.protocol_fee = protocol_fee;
            Ok(())
        }

        #[ink(message)]
        fn change_fee_receiver(
            &mut self,
            pool_key: PoolKey,
            fee_receiver: AccountId,
        ) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.state.admin {
                return Err(InvariantError::UnauthorizedAdmin);
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

            let mut token_x: contract_ref!(PSP22) = pool_key.token_x.into();
            token_x
                .transfer_from(caller, contract, x.get(), vec![])
                .map_err(|_| InvariantError::TransferError)?;
            let mut token_y: contract_ref!(PSP22) = pool_key.token_y.into();
            token_y
                .transfer_from(caller, contract, y.get(), vec![])
                .map_err(|_| InvariantError::TransferError)?;

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

            for tick in calculate_swap_result.ticks.iter() {
                self.ticks.update(pool_key, tick.index, tick);
            }

            self.pools.update(pool_key, &calculate_swap_result.pool)?;

            if x_to_y {
                let mut token_x: contract_ref!(PSP22) = pool_key.token_x.into();
                token_x
                    .transfer_from(
                        caller,
                        contract,
                        calculate_swap_result.amount_in.get(),
                        vec![],
                    )
                    .map_err(|_| InvariantError::TransferError)?;
                let mut token_y: contract_ref!(PSP22) = pool_key.token_y.into();
                token_y
                    .transfer(caller, calculate_swap_result.amount_out.get(), vec![])
                    .map_err(|_| InvariantError::TransferError)?;
            } else {
                let mut token_y: contract_ref!(PSP22) = pool_key.token_y.into();
                token_y
                    .transfer_from(
                        caller,
                        contract,
                        calculate_swap_result.amount_in.get(),
                        vec![],
                    )
                    .map_err(|_| InvariantError::TransferError)?;
                let mut token_x: contract_ref!(PSP22) = pool_key.token_x.into();
                token_x
                    .transfer(caller, calculate_swap_result.amount_out.get(), vec![])
                    .map_err(|_| InvariantError::TransferError)?;
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
            swaps: Vec<Hop>,
        ) -> Result<(), InvariantError> {
            let mut next_swap_amount = amount_in;

            for swap in swaps.iter() {
                let Hop { pool_key, x_to_y } = *swap;

                let sqrt_price_limit = if x_to_y {
                    SqrtPrice::new(MIN_SQRT_PRICE)
                } else {
                    SqrtPrice::new(MAX_SQRT_PRICE)
                };

                let result =
                    self.swap(pool_key, x_to_y, next_swap_amount, true, sqrt_price_limit)?;

                next_swap_amount = result.amount_out;
            }

            let min_amount_out = calculate_min_amount_out(expected_amount_out, slippage);

            if next_swap_amount < min_amount_out {
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
        ) -> Result<(TokenAmount, TokenAmount, SqrtPrice, Vec<Tick>), InvariantError> {
            let calculate_swap_result =
                self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

            Ok((
                calculate_swap_result.amount_in,
                calculate_swap_result.amount_out,
                calculate_swap_result.pool.sqrt_price,
                calculate_swap_result.ticks,
            ))
        }

        #[ink(message)]
        fn quote_route(
            &mut self,
            amount_in: TokenAmount,
            swaps: Vec<Hop>,
        ) -> Result<TokenAmount, InvariantError> {
            let mut next_swap_amount = amount_in;

            for swap in swaps.iter() {
                let Hop { pool_key, x_to_y } = *swap;

                let sqrt_price_limit = if x_to_y {
                    SqrtPrice::new(MIN_SQRT_PRICE)
                } else {
                    SqrtPrice::new(MAX_SQRT_PRICE)
                };

                let result = self.calculate_swap(
                    pool_key,
                    x_to_y,
                    next_swap_amount,
                    true,
                    sqrt_price_limit,
                )?;

                next_swap_amount = result.amount_out;
            }

            Ok(next_swap_amount)
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
        fn get_position(&mut self, index: u32) -> Result<Position, InvariantError> {
            let caller = self.env().caller();

            self.positions.get(caller, index)
        }

        #[ink(message)]
        fn get_all_positions(&mut self) -> Vec<Position> {
            let caller = self.env().caller();

            self.positions.get_all(caller)
        }

        #[ink(message)]
        fn update_position_seconds_per_liquidity(
            &mut self,
            index: u32,
            pool_key: PoolKey,
        ) -> Result<(), InvariantError> {
            let caller = self.env().caller();
            let current_timestamp = self.env().block_timestamp();

            let mut position = self.positions.get(caller, index)?;

            let lower_tick = self.ticks.get(pool_key, position.lower_tick_index)?;

            let upper_tick = self.ticks.get(pool_key, position.upper_tick_index)?;

            let pool = self.pools.get(pool_key)?;

            position.update_seconds_per_liquidity(
                pool,
                lower_tick,
                upper_tick,
                current_timestamp as u64,
            );
            Ok(())
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

            self.positions.update(caller, index, &position);
            self.pools.update(position.pool_key, &pool)?;
            self.ticks
                .update(position.pool_key, upper_tick.index, &upper_tick)?;
            self.ticks
                .update(position.pool_key, lower_tick.index, &lower_tick)?;

            if x.get() > 0 {
                let mut token_x: contract_ref!(PSP22) = position.pool_key.token_x.into();
                token_x
                    .transfer(caller, x.get(), vec![])
                    .map_err(|_| InvariantError::TransferError)?;
            }

            if y.get() > 0 {
                let mut token_y: contract_ref!(PSP22) = position.pool_key.token_y.into();
                token_y
                    .transfer(caller, y.get(), vec![])
                    .map_err(|_| InvariantError::TransferError)?;
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
                    current_timestamp as u64,
                    &mut lower_tick,
                    &mut upper_tick,
                    position.pool_key.fee_tier.tick_spacing,
                );

            self.pools.update(position.pool_key, pool).unwrap();

            if deinitialize_lower_tick {
                self.tickmap.flip(
                    false,
                    lower_tick.index,
                    position.pool_key.fee_tier.tick_spacing,
                    position.pool_key,
                );
                self.ticks
                    .remove(position.pool_key, position.lower_tick_index)
                    .unwrap();
            } else {
                self.ticks
                    .update(position.pool_key, position.lower_tick_index, &lower_tick)
                    .unwrap();
            }

            if deinitialize_upper_tick {
                self.tickmap.flip(
                    false,
                    upper_tick.index,
                    position.pool_key.fee_tier.tick_spacing,
                    position.pool_key,
                );
                self.ticks
                    .remove(position.pool_key, position.upper_tick_index)
                    .unwrap();
            } else {
                self.ticks
                    .update(position.pool_key, position.upper_tick_index, &upper_tick)
                    .unwrap();
            }

            self.positions.remove(caller, index).unwrap();

            let mut token_x: contract_ref!(PSP22) = position.pool_key.token_x.into();
            token_x
                .transfer(caller, amount_x.get(), vec![])
                .map_err(|_| InvariantError::TransferError)?;
            let mut token_y: contract_ref!(PSP22) = position.pool_key.token_y.into();
            token_y
                .transfer(caller, amount_y.get(), vec![])
                .map_err(|_| InvariantError::TransferError)?;

            self.emit_remove_position_event(
                caller,
                position.pool_key,
                position.liquidity,
                lower_tick.index,
                upper_tick.index,
                pool.sqrt_price,
            );
            Ok((amount_x, amount_y))
        }

        // Fee tiers
        #[ink(message)]
        fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.state.admin {
                return Err(InvariantError::UnauthorizedAdmin);
            }

            if fee_tier.tick_spacing == 0 {
                return Err(InvariantError::InvalidTickSpacing);
            }

            let fee_tier_key = FeeTierKey(fee_tier.fee, fee_tier.tick_spacing);

            if self.fee_tiers.get(fee_tier_key).is_some() {
                return Err(InvariantError::FeeTierAlreadyExist);
            } else {
                self.fee_tiers.add(fee_tier_key);
                self.fee_tier_keys.push(fee_tier_key);
                Ok(())
            }
        }

        #[ink(message)]
        fn get_fee_tier(&self, key: FeeTierKey) -> Option<()> {
            self.fee_tiers.get(key)
        }

        #[ink(message)]
        fn remove_fee_tier(&mut self, key: FeeTierKey) {
            self.fee_tiers.remove(key);
            self.fee_tier_keys.retain(|&x| x != key);
        }

        // Pools
        #[ink(message)]
        fn create_pool(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            fee_tier: FeeTier,
            init_tick: i32,
        ) -> Result<(), InvariantError> {
            let current_timestamp = self.env().block_timestamp();

            let fee_tier_key = FeeTierKey(fee_tier.fee, fee_tier.tick_spacing);
            self.fee_tiers
                .get(fee_tier_key)
                .ok_or(InvariantError::FeeTierNotFound)?;

            let pool_key = PoolKey::new(token_0, token_1, fee_tier)?;
            let pool = Pool::create(init_tick, current_timestamp, self.state.admin);
            self.pools.add(pool_key, &pool)?;

            self.pool_keys.push(pool_key);

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

        fn remove_pool(&mut self, key: PoolKey) {
            self.pools.remove(key);
            self.pool_keys.retain(|&x| x != key);
        }

        // Ticks
        fn add_tick(&mut self, key: PoolKey, index: i32, tick: Tick) {
            self.ticks.add(key, index, &tick);
        }

        #[ink(message)]
        fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
            self.ticks.get(key, index)
        }

        #[ink(message)]
        fn get_tickmap_bit(&self, key: PoolKey, index: i32) -> bool {
            self.tickmap.get(index, key.fee_tier.tick_spacing, key)
        }
        fn remove_tick(&mut self, key: PoolKey, index: i32) {
            self.ticks.remove(key, index);
        }

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
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                SwapEvent {
                    timestamp,
                    address,
                    pool,
                    amount_in,
                    amount_out,
                    fee,
                    start_sqrt_price,
                    target_sqrt_price,
                    x_to_y,
                },
            );
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
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                CreatePositionEvent {
                    timestamp,
                    address,
                    pool,
                    liquidity,
                    lower_tick,
                    upper_tick,
                    current_sqrt_price,
                },
            );
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
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                RemovePositionEvent {
                    timestamp,
                    address,
                    pool,
                    liquidity,
                    lower_tick,
                    upper_tick,
                    current_sqrt_price,
                },
            );
        }
        fn emit_cross_tick_event(&self, address: AccountId, pool: PoolKey, index: i32) {
            let timestamp = self.get_timestamp();
            ink::codegen::EmitEvent::<Contract>::emit_event(
                self.env(),
                CrossTickEvent {
                    timestamp,
                    address,
                    pool,
                    index,
                },
            );
        }

        fn get_timestamp(&self) -> u64 {
            self.env().block_timestamp()
        }

        fn _order_tokens(
            &self,
            token_0: AccountId,
            token_1: AccountId,
            balance_0: Balance,
            balance_1: Balance,
        ) -> OrderPair {
            match token_0.lt(&token_1) {
                true => OrderPair {
                    x: (token_0, balance_0),
                    y: (token_1, balance_1),
                },
                false => OrderPair {
                    x: (token_1, balance_1),
                    y: (token_0, balance_0),
                },
            }
        }
    }

    #[cfg(test)]
    mod tests {

        use super::*;

        use crate::math::consts::MAX_TICK;
        use crate::math::percentage::Percentage;

        #[ink::test]
        fn initialize_works() {
            let _ = Contract::new(Percentage::new(0));
        }

        #[ink::test]
        fn test_add_pool() {
            let mut contract = Contract::new(Percentage::new(0));
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 1,
            };

            contract.add_fee_tier(fee_tier).unwrap();

            let result = contract.create_pool(
                token_0,
                token_1,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
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
                0,
            );
            assert_eq!(result, Err(InvariantError::PoolAlreadyExist));
        }

        #[ink::test]
        fn test_get_pool() {
            let mut contract = Contract::new(Percentage::new(0));
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
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

            let result = contract.create_pool(token_0, token_1, fee_tier, 0);
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
            let mut contract = Contract::new(Percentage::new(0));
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
            let _ = contract.create_pool(pool_key.token_x, pool_key.token_y, pool_key.fee_tier, 0);
            let result = contract.create_tick(pool_key, 0);
            assert!(result.is_ok());
        }

        #[ink::test]
        fn test_fee_tiers() {
            let mut contract = Contract::new(Percentage::new(0));
            let fee_tier_key = FeeTierKey(Percentage::new(1), 10u16);
            let fee_tier_value = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 10u16,
            };

            contract.add_fee_tier(fee_tier_value).unwrap();
            assert_eq!(contract.fee_tier_keys.len(), 1);
            contract.add_fee_tier(fee_tier_value).unwrap_err();
            contract.remove_fee_tier(fee_tier_key);
            assert_eq!(contract.fee_tier_keys.len(), 0);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    pub mod e2e_tests {
        use crate::contracts::{get_liquidity, get_liquidity_by_x, get_liquidity_by_y};
        use crate::math::fee_growth::FeeGrowth;
        use crate::math::get_delta_y;
        use crate::math::sqrt_price::log::get_tick_at_sqrt_price;
        use crate::math::sqrt_price::sqrt_price::{calculate_sqrt_price, get_max_tick};
        use crate::math::MAX_TICK;
        use ink::prelude::vec;
        use ink::prelude::vec::Vec;
        use ink_e2e::build_message;

        use test_helpers::{
            address_of, approve, balance_of, big_deposit_and_swap, change_fee_receiver, claim_fee,
            create_3_tokens, create_dex, create_fee_tier, create_pool, create_position,
            create_slippage_pool_with_liquidity, create_standard_fee_tiers, create_tokens,
            dex_balance, get_all_positions, get_fee_tier, get_pool, get_position, get_tick,
            init_basic_pool, init_basic_position, init_basic_swap, init_cross_position,
            init_cross_swap, init_dex_and_3_tokens, init_dex_and_tokens,
            init_dex_and_tokens_max_mint_amount, init_slippage_dex_and_tokens, mint,
            mint_with_aprove_for_bob, multiple_swap, quote, quote_route, remove_position, swap,
            swap_exact_limit, swap_route, tickmap_bit, withdraw_protocol_fee,
        };
        use token::TokenRef;

        use super::*;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn swap_route(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y, token_z) =
                init_dex_and_3_tokens!(client, ContractRef, TokenRef);

            let alice = ink_e2e::alice();
            approve!(client, TokenRef, token_x, dex, u64::MAX as u128, alice);
            approve!(client, TokenRef, token_y, dex, u64::MAX as u128, alice);
            approve!(client, TokenRef, token_z, dex, u64::MAX as u128, alice);

            let amount = 1000;
            let bob = ink_e2e::bob();
            mint_with_aprove_for_bob!(client, TokenRef, token_x, dex, amount);
            approve!(client, TokenRef, token_y, dex, u64::MAX as u128, bob);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1);

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let init_tick = 0;
            create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let init_tick = 0;
            create_pool!(
                client,
                ContractRef,
                dex,
                token_y,
                token_z,
                fee_tier,
                init_tick
            );

            let pool_key_1 = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let pool_key_2 = PoolKey::new(token_y, token_z, fee_tier).unwrap();

            let liquidity_delta = Liquidity::new(2u128.pow(63) - 1);

            let pool_1 = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let slippage_limit_lower = pool_1.sqrt_price;
            let slippage_limit_upper = pool_1.sqrt_price;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key_1,
                -1,
                1,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                alice
            );

            let pool_2 = get_pool!(client, ContractRef, dex, token_y, token_z, fee_tier).unwrap();
            let slippage_limit_lower = pool_2.sqrt_price;
            let slippage_limit_upper = pool_2.sqrt_price;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key_2,
                -1,
                1,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                alice
            );

            let amount_in = TokenAmount(1000);
            let expected_amount_out = TokenAmount(1000);
            let slippage = Percentage::new(0);
            let swaps = vec![
                Hop {
                    pool_key: pool_key_1,
                    x_to_y: true,
                },
                Hop {
                    pool_key: pool_key_2,
                    x_to_y: true,
                },
            ];

            let expected_token_amount =
                quote_route!(client, ContractRef, dex, amount_in, swaps.clone(), bob).unwrap();

            swap_route!(
                client,
                ContractRef,
                dex,
                amount_in,
                expected_token_amount,
                slippage,
                swaps.clone(),
                bob
            );

            let bob_amount_x = balance_of!(TokenRef, client, token_x, Bob);
            let bob_amount_y = balance_of!(TokenRef, client, token_y, Bob);
            let bob_amount_z = balance_of!(TokenRef, client, token_z, Bob);

            assert_eq!(bob_amount_x, 0);
            assert_eq!(bob_amount_y, 0);
            assert_eq!(bob_amount_z, 986);

            let pool_1_after =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            assert_eq!(pool_1_after.fee_protocol_token_x, TokenAmount(1));
            assert_eq!(pool_1_after.fee_protocol_token_y, TokenAmount(0));

            let pool_2_after =
                get_pool!(client, ContractRef, dex, token_y, token_z, fee_tier).unwrap();
            assert_eq!(pool_2_after.fee_protocol_token_x, TokenAmount(1));
            assert_eq!(pool_2_after.fee_protocol_token_y, TokenAmount(0));

            let alice_amount_x_before = balance_of!(TokenRef, client, token_x, Alice);
            let alice_amount_y_before = balance_of!(TokenRef, client, token_y, Alice);
            let alice_amount_z_before = balance_of!(TokenRef, client, token_z, Alice);

            claim_fee!(client, ContractRef, dex, 0, alice);
            claim_fee!(client, ContractRef, dex, 1, alice);

            let alice_amount_x_after = balance_of!(TokenRef, client, token_x, Alice);
            let alice_amount_y_after = balance_of!(TokenRef, client, token_y, Alice);
            let alice_amount_z_after = balance_of!(TokenRef, client, token_z, Alice);

            assert_eq!(alice_amount_x_after - alice_amount_x_before, 4);
            assert_eq!(alice_amount_y_after - alice_amount_y_before, 4);
            assert_eq!(alice_amount_z_after - alice_amount_z_before, 0);

            Ok(())
        }

        #[ink_e2e::test]
        async fn limits_full_range_with_max_liquidity(mut client: ink_e2e::Client<C, E>) -> () {
            let (dex, token_x, token_y) =
                init_dex_and_tokens_max_mint_amount!(client, ContractRef, TokenRef);

            let mint_amount = u128::MAX;
            let alice = ink_e2e::alice();
            approve!(client, TokenRef, token_x, dex, u128::MAX, alice);
            approve!(client, TokenRef, token_y, dex, u128::MAX, alice);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1);
            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let init_tick = get_max_tick(1);
            create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let current_sqrt_price = pool.sqrt_price;
            assert_eq!(pool.current_tick_index, init_tick);
            assert_eq!(pool.sqrt_price, calculate_sqrt_price(init_tick).unwrap());

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let liquidity_delta = Liquidity::new(2u128.pow(109) - 1);
            let slippage_limit_lower = pool.sqrt_price;
            let slippage_limit_upper = pool.sqrt_price;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -MAX_TICK,
                MAX_TICK,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                alice
            );

            let contract_amount_x = dex_balance!(TokenRef, client, token_x, dex);
            let contract_amount_y = dex_balance!(TokenRef, client, token_y, dex);

            let expected_x = 0;
            let expected_y = 42534896005851865508212194815854;
            assert_eq!(contract_amount_x, expected_x);
            assert_eq!(contract_amount_y, expected_y);
        }

        #[ink_e2e::test]
        async fn deposit_limits_at_upper_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) =
                init_dex_and_tokens_max_mint_amount!(client, ContractRef, TokenRef);

            let mint_amount = 2u128.pow(105) - 1;
            let alice = ink_e2e::alice();
            approve!(client, TokenRef, token_x, dex, u128::MAX, alice);
            approve!(client, TokenRef, token_y, dex, u128::MAX, alice);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1);
            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let init_tick = get_max_tick(1);
            create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let current_sqrt_price = pool.sqrt_price;
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

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let slippage_limit_lower = pool.sqrt_price;
            let slippage_limit_upper = pool.sqrt_price;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                0,
                MAX_TICK,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                alice
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn limits_big_deposit_and_swaps(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) =
                init_dex_and_tokens_max_mint_amount!(client, ContractRef, TokenRef);

            let mint_amount = 2u128.pow(76) - 1;
            let alice = ink_e2e::alice();
            approve!(client, TokenRef, token_x, dex, u128::MAX, alice);
            approve!(client, TokenRef, token_y, dex, u128::MAX, alice);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1);
            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let init_tick = 0;
            create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let pos_amount = mint_amount / 2;
            let lower_tick = -(fee_tier.tick_spacing as i32);
            let upper_tick = fee_tier.tick_spacing as i32;
            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

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

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let slippage_limit_lower = pool.sqrt_price;
            let slippage_limit_upper = pool.sqrt_price;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick,
                upper_tick,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                alice
            );

            let user_amount_x = balance_of!(TokenRef, client, token_x, Alice);
            let user_amount_y = balance_of!(TokenRef, client, token_y, Alice);
            assert_eq!(user_amount_x, u128::MAX - pos_amount);
            assert_eq!(user_amount_y, u128::MAX - y.get());

            let contract_amount_x = dex_balance!(TokenRef, client, token_x, dex);
            let contract_amount_y = dex_balance!(TokenRef, client, token_y, dex);
            assert_eq!(contract_amount_x, pos_amount);
            assert_eq!(contract_amount_y, y.get());

            let swap_amount = TokenAmount(mint_amount / 8);

            for i in 1..=4 {
                let (x_to_y, sqrt_price_limit) = if i % 2 == 0 {
                    (true, SqrtPrice::new(MIN_SQRT_PRICE))
                } else {
                    (false, SqrtPrice::new(MAX_SQRT_PRICE))
                };

                swap!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    i % 2 == 0,
                    swap_amount,
                    true,
                    sqrt_price_limit,
                    alice
                );
            }

            Ok(())
        }

        #[ink_e2e::test]
        async fn multiple_swap_x_to_y(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            multiple_swap!(client, ContractRef, TokenRef, true);
            Ok(())
        }

        #[ink_e2e::test]
        async fn limits_big_deposit_x_and_swap_y(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            big_deposit_and_swap!(client, ContractRef, TokenRef, true);

            Ok(())
        }

        #[ink_e2e::test]
        async fn multiple_swap_y_to_x(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            multiple_swap!(client, ContractRef, TokenRef, false);
            Ok(())
        }

        #[ink_e2e::test]
        async fn limits_big_deposit_y_and_swap_x(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            big_deposit_and_swap!(client, ContractRef, TokenRef, false);

            Ok(())
        }

        #[ink_e2e::test]
        async fn limits_big_deposit_both_tokens(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let (dex, token_x, token_y) =
                init_dex_and_tokens_max_mint_amount!(client, ContractRef, TokenRef);

            let mint_amount = 2u128.pow(75) - 1;
            let alice = ink_e2e::alice();
            approve!(client, TokenRef, token_x, dex, u128::MAX, alice);
            approve!(client, TokenRef, token_y, dex, u128::MAX, alice);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1);

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let init_tick = 0;
            create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let lower_tick = -(fee_tier.tick_spacing as i32);
            let upper_tick = fee_tier.tick_spacing as i32;
            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
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

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let slippage_limit_lower = pool.sqrt_price;
            let slippage_limit_upper = pool.sqrt_price;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick,
                upper_tick,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                alice
            );

            let user_amount_x = balance_of!(TokenRef, client, token_x, Alice);
            let user_amount_y = balance_of!(TokenRef, client, token_y, Alice);
            assert_eq!(user_amount_x, u128::MAX - mint_amount);
            assert_eq!(user_amount_y, u128::MAX - y.get());

            let contract_amount_x = dex_balance!(TokenRef, client, token_x, dex);
            let contract_amount_y = dex_balance!(TokenRef, client, token_y, dex);
            assert_eq!(contract_amount_x, mint_amount);
            assert_eq!(contract_amount_y, y.get());

            Ok(())
        }

        #[ink_e2e::test]
        async fn max_tick_cross(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
            init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);

            let mint_amount = u128::MAX;
            let alice = ink_e2e::alice();
            approve!(client, TokenRef, token_x, dex, mint_amount, alice);
            approve!(client, TokenRef, token_y, dex, mint_amount, alice);

            let liquidity = Liquidity::from_integer(10000000);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            for i in (-2560..20).step_by(10) {
                let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

                let slippage_limit_lower = pool.sqrt_price;
                let slippage_limit_upper = pool.sqrt_price;

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    i,
                    i + 10,
                    liquidity,
                    slippage_limit_lower,
                    slippage_limit_upper,
                    alice
                );
            }

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            assert_eq!(pool.liquidity, liquidity);

            let amount = 760_000;
            let bob = ink_e2e::bob();
            mint!(TokenRef, client, token_x, Bob, amount);
            let amount_x = balance_of!(TokenRef, client, token_x, Bob);
            assert_eq!(amount_x, amount);
            approve!(client, TokenRef, token_x, dex, amount, bob);

            let pool_before = get_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                pool_key.fee_tier
            )
            .unwrap();

            let swap_amount = TokenAmount::new(amount);
            let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
            let quote_result = quote!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                slippage,
                bob
            )
            .unwrap();

            let pool_after_quote = get_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                pool_key.fee_tier
            )
            .unwrap();

            let crosses_after_quote =
                ((pool_after_quote.current_tick_index - pool_before.current_tick_index) / 10).abs();
            assert_eq!(crosses_after_quote, 0);
            assert_eq!(quote_result.3.len() - 1, 146);

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                slippage,
                bob
            );

            let pool_after = get_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                pool_key.fee_tier
            )
            .unwrap();

            let crosses =
                ((pool_after.current_tick_index - pool_before.current_tick_index) / 10).abs();
            assert_eq!(crosses, 146);
            assert_eq!(
                pool_after.current_tick_index,
                get_tick_at_sqrt_price(quote_result.2, 10).unwrap()
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn swap_exact_limit(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
            init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            let amount = 1000;
            let bob = ink_e2e::bob();
            mint!(TokenRef, client, token_x, Bob, amount);
            let amount_x = balance_of!(TokenRef, client, token_x, Bob);
            assert_eq!(amount_x, amount);
            approve!(client, TokenRef, token_x, dex, amount, bob);

            let swap_amount = TokenAmount::new(amount);
            swap_exact_limit!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                bob
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn claim(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
            init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_swap!(client, ContractRef, TokenRef, dex, token_x, token_y);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let alice = ink_e2e::alice();
            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let user_amount_before_claim = balance_of!(TokenRef, client, token_x, Alice);
            let dex_amount_before_claim = dex_balance!(TokenRef, client, token_x, dex);

            claim_fee!(client, ContractRef, dex, 0, alice);

            let user_amount_after_claim = balance_of!(TokenRef, client, token_x, Alice);
            let dex_amount_after_claim = dex_balance!(TokenRef, client, token_x, dex);
            let position = get_position!(client, ContractRef, dex, 0, alice).unwrap();
            let expected_tokens_claimed = 5;

            assert_eq!(
                user_amount_after_claim - expected_tokens_claimed,
                user_amount_before_claim
            );
            assert_eq!(
                dex_amount_after_claim + expected_tokens_claimed,
                dex_amount_before_claim
            );
            assert_eq!(position.fee_growth_inside_x, pool.fee_growth_global_x);
            assert_eq!(position.tokens_owed_x, TokenAmount(0));

            Ok(())
        }

        #[ink_e2e::test]
        async fn basic_slippage_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let (dex, token_x, token_y) =
                init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
            let pool_key = create_slippage_pool_with_liquidity!(
                client,
                ContractRef,
                TokenRef,
                dex,
                token_x,
                token_y
            );
            let amount = 10u128.pow(8);
            let swap_amount = TokenAmount::new(amount);
            approve!(client, TokenRef, token_x, dex, amount, alice);

            let target_sqrt_price = SqrtPrice::new(1009940000000000000000001);
            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                false,
                swap_amount,
                true,
                target_sqrt_price,
                alice
            );
            let expected_sqrt_price = SqrtPrice::new(1009940000000000000000000);
            let pool = get_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                pool_key.fee_tier
            )
            .unwrap();

            assert_eq!(expected_sqrt_price, pool.sqrt_price);
            Ok(())
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn swap_close_to_limit_test(mut client: ink_e2e::Client<C, E>) -> () {
            let alice = ink_e2e::alice();
            let (dex, token_x, token_y) =
                init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
            let pool_key = create_slippage_pool_with_liquidity!(
                client,
                ContractRef,
                TokenRef,
                dex,
                token_x,
                token_y
            );
            let amount = 10u128.pow(8);
            let swap_amount = TokenAmount::new(amount);
            approve!(client, TokenRef, token_x, dex, amount, alice);

            let target_sqrt_price = calculate_sqrt_price(-98).unwrap();
            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                false,
                swap_amount,
                true,
                target_sqrt_price,
                alice
            );
        }

        #[ink_e2e::test]
        async fn cross(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
            init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_cross_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_cross_swap!(client, ContractRef, TokenRef, dex, token_x, token_y);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let alice = ink_e2e::alice();

            let upper_tick_index = 10;
            let middle_tick_index = -10;
            let lower_tick_index = -20;

            let upper_tick =
                get_tick!(client, ContractRef, dex, upper_tick_index, pool_key, alice).unwrap();
            let middle_tick =
                get_tick!(client, ContractRef, dex, middle_tick_index, pool_key, alice).unwrap();
            let lower_tick =
                get_tick!(client, ContractRef, dex, lower_tick_index, pool_key, alice).unwrap();

            assert_eq!(
                upper_tick.liquidity_change,
                Liquidity::from_integer(1000000)
            );
            assert_eq!(
                middle_tick.liquidity_change,
                Liquidity::from_integer(1000000)
            );
            assert_eq!(
                lower_tick.liquidity_change,
                Liquidity::from_integer(1000000)
            );

            assert_eq!(upper_tick.fee_growth_outside_x, FeeGrowth::new(0));
            assert_eq!(
                middle_tick.fee_growth_outside_x,
                FeeGrowth::new(30000000000000000000000)
            );
            assert_eq!(lower_tick.fee_growth_outside_x, FeeGrowth::new(0));

            Ok(())
        }

        #[ink_e2e::test]
        async fn swap(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
            init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_swap!(client, ContractRef, TokenRef, dex, token_x, token_y);
            Ok(())
        }

        #[ink_e2e::test]
        async fn protocol_fee(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
            init_basic_pool!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_swap!(client, ContractRef, TokenRef, dex, token_x, token_y);

            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let alice = ink_e2e::alice();
            withdraw_protocol_fee!(client, ContractRef, dex, pool_key, alice);

            let amount_x = balance_of!(TokenRef, client, token_x, Alice);
            let amount_y = balance_of!(TokenRef, client, token_y, Alice);
            assert_eq!(amount_x, 9999999501);
            assert_eq!(amount_y, 9999999000);

            let amount_x = dex_balance!(TokenRef, client, token_x, dex);
            let amount_y = dex_balance!(TokenRef, client, token_y, dex);
            assert_eq!(amount_x, 1499);
            assert_eq!(amount_y, 7);

            let pool_after_withdraw =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            assert_eq!(
                pool_after_withdraw.fee_protocol_token_x,
                TokenAmount::new(0)
            );
            assert_eq!(
                pool_after_withdraw.fee_protocol_token_y,
                TokenAmount::new(0)
            );

            Ok(())
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn protocol_fee_should_panic(mut client: ink_e2e::Client<C, E>) -> () {
            let (dex, token_x, token_y) = init_dex_and_tokens!(client, ContractRef, TokenRef);
            init_basic_position!(client, ContractRef, TokenRef, dex, token_x, token_y);
            init_basic_swap!(client, ContractRef, TokenRef, dex, token_x, token_y);

            let pool_key = PoolKey::new(
                token_x,
                token_y,
                FeeTier {
                    fee: Percentage::from_scale(6, 3),
                    tick_spacing: 10,
                },
            )
            .unwrap();
            let bob = ink_e2e::bob();
            withdraw_protocol_fee!(client, ContractRef, dex, pool_key, bob);
        }

        #[ink_e2e::test]
        async fn constructor_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500, None, None, 0);
            let _token: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));

            let _contract: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;
            Ok(())
        }

        #[ink_e2e::test]
        async fn change_protocol_fee(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let contract = create_dex!(client, ContractRef, Percentage::new(0));

            let protocol_fee = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_protocol_fee());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("getting protocol fee failed")
            }
            .return_value();

            assert_eq!(protocol_fee, Percentage::new(0));

            let _result = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.change_protocol_fee(Percentage::new(1)));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("changing protocol fee failed")
            };

            let protocol_fee = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_protocol_fee());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("getting protocol fee failed")
            }
            .return_value();

            assert_eq!(protocol_fee, Percentage::new(1));

            Ok(())
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn change_protocol_fee_should_panic(mut client: ink_e2e::Client<C, E>) -> () {
            let contract = create_dex!(client, ContractRef, Percentage::new(0));

            let result = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.change_protocol_fee(Percentage::new(1)));
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("changing protocol fee failed")
            };
        }

        #[ink_e2e::test]
        async fn create_position(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let (token_x, token_y) = create_tokens!(client, TokenRef, TokenRef, 500, 500);

            let alice = ink_e2e::alice();

            let fee_tier = FeeTier::new(Percentage::new(0), 1);

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(client, ContractRef, dex, token_x, token_y, fee_tier, 10);

            approve!(client, TokenRef, token_x, dex, 500, alice);
            approve!(client, TokenRef, token_y, dex, 500, alice);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            let position = create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -10,
                10,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn create_fee_tier_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let fee_tier = FeeTier::new(Percentage::new(0), 10u16);
            let alice = ink_e2e::alice();
            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);
            let fee_tier = get_fee_tier!(client, ContractRef, dex, Percentage::new(0), 10u16);
            assert!(fee_tier.is_some());
            Ok(())
        }

        #[ink_e2e::test]
        async fn create_standard_fee_tier_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            create_standard_fee_tiers!(client, ContractRef, dex);
            let fee_tier = get_fee_tier!(
                client,
                ContractRef,
                dex,
                Percentage::from_scale(5, 2),
                100u16
            );
            assert!(fee_tier.is_some());
            Ok(())
        }

        #[ink_e2e::test]
        async fn create_pool_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let (token_x, token_y) = create_tokens!(client, TokenRef, TokenRef, 500, 500);

            let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100);
            let init_tick = 0;

            let alice = ink_e2e::alice();

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let result = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );
            assert!(result.is_ok());

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            Ok(())
        }

        #[ink_e2e::test]
        async fn fee_tier_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let admin = ink_e2e::alice();
            let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100);
            let result = create_fee_tier!(client, ContractRef, dex, fee_tier, admin);
            assert!(result.is_ok());
            Ok(())
        }
        #[ink_e2e::test]
        #[should_panic]
        async fn invalid_spacing_fee_tier_test(mut client: ink_e2e::Client<C, E>) -> () {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let admin = ink_e2e::alice();
            // 0 tick spacing | should fail
            let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 0);
            let result = create_fee_tier!(client, ContractRef, dex, fee_tier, admin);
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn non_admin_fee_tier_caller_test(mut client: ink_e2e::Client<C, E>) -> () {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let user = ink_e2e::bob();
            // not-admin
            let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10);
            let result = create_fee_tier!(client, ContractRef, dex, fee_tier, user);
        }

        #[ink_e2e::test]
        async fn position_above_current_tick_test(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let init_tick = -23028;

            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let initial_balance = 10_000_000_000;

            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_balance, initial_balance);

            let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4);

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            approve!(client, TokenRef, token_x, dex, initial_balance, alice);
            approve!(client, TokenRef, token_y, dex, initial_balance, alice);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let lower_tick_index = -22980;
            let upper_tick_index = 0;
            let liquidity_delta = Liquidity::new(initial_balance);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            );

            // Load states
            let position_state = get_position!(client, ContractRef, dex, 0, alice).unwrap();
            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let lower_tick =
                get_tick!(client, ContractRef, dex, lower_tick_index, pool_key, alice).unwrap();
            let upper_tick =
                get_tick!(client, ContractRef, dex, upper_tick_index, pool_key, alice).unwrap();
            let lower_tick_bit =
                tickmap_bit!(client, ContractRef, dex, lower_tick_index, pool_key, alice);
            let upper_tick_bit =
                tickmap_bit!(client, ContractRef, dex, upper_tick_index, pool_key, alice);
            let alice_x = balance_of!(TokenRef, client, token_x, Alice);
            let alice_y = balance_of!(TokenRef, client, token_y, Alice);
            let dex_x = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y = dex_balance!(TokenRef, client, token_y, dex);

            let zero_fee = FeeGrowth::new(0);
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

        #[ink_e2e::test]
        async fn multiple_positions_on_same_tick(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let init_tick = 0;

            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let initial_balance = 100_000_000;

            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_balance, initial_balance);

            let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 10);

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            approve!(client, TokenRef, token_x, dex, initial_balance, alice);
            approve!(client, TokenRef, token_y, dex, initial_balance, alice);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            // Three position on same lower and upper tick
            {
                let lower_tick_index = -10;
                let upper_tick_index = 10;

                let liquidity_delta = Liquidity::new(100);

                let pool_state =
                    get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    lower_tick_index,
                    upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    SqrtPrice::max_instance(),
                    alice
                );

                let first_position = get_position!(client, ContractRef, dex, 0, alice).unwrap();

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    lower_tick_index,
                    upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    SqrtPrice::max_instance(),
                    alice
                );

                let second_position = get_position!(client, ContractRef, dex, 1, alice).unwrap();

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    lower_tick_index,
                    upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    SqrtPrice::max_instance(),
                    alice
                );

                let third_position = get_position!(client, ContractRef, dex, 2, alice).unwrap();

                assert!(first_position.lower_tick_index == second_position.lower_tick_index);
                assert!(first_position.upper_tick_index == second_position.upper_tick_index);
                assert!(first_position.lower_tick_index == third_position.lower_tick_index);
                assert!(first_position.upper_tick_index == third_position.upper_tick_index);

                // Load states
                let pool_state =
                    get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
                let lower_tick =
                    get_tick!(client, ContractRef, dex, lower_tick_index, pool_key, alice).unwrap();
                let upper_tick =
                    get_tick!(client, ContractRef, dex, upper_tick_index, pool_key, alice).unwrap();
                let lower_tick_bit =
                    tickmap_bit!(client, ContractRef, dex, lower_tick_index, pool_key, alice);
                let upper_tick_bit =
                    tickmap_bit!(client, ContractRef, dex, upper_tick_index, pool_key, alice);
                let expected_liquidity = Liquidity::new(liquidity_delta.get() * 3);
                let zero_fee = FeeGrowth::new(0);

                // Check ticks
                assert!(lower_tick.index == lower_tick_index);
                assert!(upper_tick.index == upper_tick_index);
                assert_eq!(lower_tick.liquidity_gross, expected_liquidity);
                assert_eq!(upper_tick.liquidity_gross, expected_liquidity);
                assert_eq!(lower_tick.liquidity_change, expected_liquidity);
                assert_eq!(upper_tick.liquidity_change, expected_liquidity);
                assert!(lower_tick.sign);
                assert!(!upper_tick.sign);

                // Check pool
                assert_eq!(pool_state.liquidity, expected_liquidity);
                assert!(pool_state.current_tick_index == init_tick);

                // Check first position
                assert!(first_position.pool_key == pool_key);
                assert!(first_position.liquidity == liquidity_delta);
                assert!(first_position.lower_tick_index == lower_tick_index);
                assert!(first_position.upper_tick_index == upper_tick_index);
                assert!(first_position.fee_growth_inside_x == zero_fee);
                assert!(first_position.fee_growth_inside_y == zero_fee);

                // Check second position
                assert!(second_position.pool_key == pool_key);
                assert!(second_position.liquidity == liquidity_delta);
                assert!(second_position.lower_tick_index == lower_tick_index);
                assert!(second_position.upper_tick_index == upper_tick_index);
                assert!(second_position.fee_growth_inside_x == zero_fee);
                assert!(second_position.fee_growth_inside_y == zero_fee);

                // Check third position
                assert!(third_position.pool_key == pool_key);
                assert!(third_position.liquidity == liquidity_delta);
                assert!(third_position.lower_tick_index == lower_tick_index);
                assert!(third_position.upper_tick_index == upper_tick_index);
                assert!(third_position.fee_growth_inside_x == zero_fee);
                assert!(third_position.fee_growth_inside_y == zero_fee);
            }
            {
                let lower_tick_index = -10;
                let upper_tick_index = 10;
                let zero_fee = FeeGrowth::new(0);

                let liquidity_delta = Liquidity::new(100);

                let pool_state =
                    get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    lower_tick_index,
                    upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    SqrtPrice::max_instance(),
                    alice
                );

                let first_position = get_position!(client, ContractRef, dex, 3, alice).unwrap();

                // Check first position
                assert!(first_position.pool_key == pool_key);
                assert!(first_position.liquidity == liquidity_delta);
                assert!(first_position.lower_tick_index == lower_tick_index);
                assert!(first_position.upper_tick_index == upper_tick_index);
                assert!(first_position.fee_growth_inside_x == zero_fee);
                assert!(first_position.fee_growth_inside_y == zero_fee);

                let lower_tick_index = -20;
                let upper_tick_index = -10;

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    lower_tick_index,
                    upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    SqrtPrice::max_instance(),
                    alice
                );

                let second_position = get_position!(client, ContractRef, dex, 4, alice).unwrap();

                // Check second position
                assert!(second_position.pool_key == pool_key);
                assert!(second_position.liquidity == liquidity_delta);
                assert!(second_position.lower_tick_index == lower_tick_index);
                assert!(second_position.upper_tick_index == upper_tick_index);
                assert!(second_position.fee_growth_inside_x == zero_fee);
                assert!(second_position.fee_growth_inside_y == zero_fee);

                let lower_tick_index = 10;
                let upper_tick_index = 20;
                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    lower_tick_index,
                    upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    SqrtPrice::max_instance(),
                    alice
                );

                let third_position = get_position!(client, ContractRef, dex, 5, alice).unwrap();

                // Check third position
                assert!(third_position.pool_key == pool_key);
                assert!(third_position.liquidity == liquidity_delta);
                assert!(third_position.lower_tick_index == lower_tick_index);
                assert!(third_position.upper_tick_index == upper_tick_index);
                assert!(third_position.fee_growth_inside_x == zero_fee);
                assert!(third_position.fee_growth_inside_y == zero_fee);

                // Load states
                let pool_state =
                    get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
                let tick_n20 = get_tick!(client, ContractRef, dex, -20, pool_key, alice).unwrap();
                let tick_n10 = get_tick!(client, ContractRef, dex, -10, pool_key, alice).unwrap();
                let tick_10 = get_tick!(client, ContractRef, dex, 10, pool_key, alice).unwrap();
                let tick_20 = get_tick!(client, ContractRef, dex, 20, pool_key, alice).unwrap();
                let tick_n20_bit = tickmap_bit!(client, ContractRef, dex, -20, pool_key, alice);
                let tick_n10_bit = tickmap_bit!(client, ContractRef, dex, -10, pool_key, alice);
                let tick_10_bit = tickmap_bit!(client, ContractRef, dex, 10, pool_key, alice);
                let tick_20_bit = tickmap_bit!(client, ContractRef, dex, 20, pool_key, alice);

                let expected_active_liquidity = Liquidity::new(400);

                // Check tick -20
                assert_eq!(tick_n20.index, -20);
                assert_eq!(tick_n20.liquidity_gross, Liquidity::new(100));
                assert_eq!(tick_n20.liquidity_change, Liquidity::new(100));
                assert!(tick_n20.sign);
                assert!(tick_n20_bit);

                // Check tick -10
                assert_eq!(tick_n10.index, -10);
                assert_eq!(tick_n10.liquidity_gross, Liquidity::new(500));
                assert_eq!(tick_n10.liquidity_change, Liquidity::new(300));
                assert!(tick_n10.sign);
                assert!(tick_n10_bit);

                // Check tick 10
                assert_eq!(tick_10.index, 10);
                assert_eq!(tick_10.liquidity_gross, Liquidity::new(500));
                assert_eq!(tick_10.liquidity_change, Liquidity::new(300));
                assert!(!tick_10.sign);
                assert!(tick_20_bit);

                // Check tick 20
                assert_eq!(tick_20.index, 20);
                assert_eq!(tick_20.liquidity_gross, Liquidity::new(100));
                assert_eq!(tick_20.liquidity_change, Liquidity::new(100));
                assert!(!tick_20.sign);
                assert!(tick_20_bit);

                // Check pool
                assert_eq!(pool_state.liquidity, expected_active_liquidity);
                assert!(pool_state.current_tick_index == init_tick);
            }
            Ok(())
        }

        #[ink_e2e::test]
        async fn position_within_current_tick_test(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let MAX_TICK_TEST = 177_450; // for tickSpacing 4
            let MIN_TICK_TEST = -MAX_TICK_TEST;
            let alice = ink_e2e::alice();
            let init_tick = -23028;

            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let initial_balance = 100_000_000;

            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_balance, initial_balance);

            let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4);

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            approve!(client, TokenRef, token_x, dex, initial_balance, alice);
            approve!(client, TokenRef, token_y, dex, initial_balance, alice);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let lower_tick_index = MIN_TICK_TEST + 10;
            let upper_tick_index = MAX_TICK_TEST - 10;

            let liquidity_delta = Liquidity::new(initial_balance);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            );

            // Load states
            let position_state = get_position!(client, ContractRef, dex, 0, alice).unwrap();
            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let lower_tick =
                get_tick!(client, ContractRef, dex, lower_tick_index, pool_key, alice).unwrap();
            let upper_tick =
                get_tick!(client, ContractRef, dex, upper_tick_index, pool_key, alice).unwrap();
            let lower_tick_bit =
                tickmap_bit!(client, ContractRef, dex, lower_tick_index, pool_key, alice);
            let upper_tick_bit =
                tickmap_bit!(client, ContractRef, dex, upper_tick_index, pool_key, alice);
            let alice_x = balance_of!(TokenRef, client, token_x, Alice);
            let alice_y = balance_of!(TokenRef, client, token_y, Alice);
            let dex_x = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y = dex_balance!(TokenRef, client, token_y, dex);

            let zero_fee = FeeGrowth::new(0);
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
        async fn position_below_current_tick_test(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let init_tick = -23028;

            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let initial_balance = 100_000_000_00;

            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_balance, initial_balance);

            let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4);

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            approve!(client, TokenRef, token_x, dex, initial_balance, alice);
            approve!(client, TokenRef, token_y, dex, initial_balance, alice);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            let lower_tick_index = -46080;
            let upper_tick_index = -23040;

            let liquidity_delta = Liquidity::new(initial_balance);

            let pool_state_before =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state_before.sqrt_price,
                SqrtPrice::max_instance(),
                alice
            );

            // Load states
            let position_state = get_position!(client, ContractRef, dex, 0, alice).unwrap();
            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let lower_tick =
                get_tick!(client, ContractRef, dex, lower_tick_index, pool_key, alice).unwrap();
            let upper_tick =
                get_tick!(client, ContractRef, dex, upper_tick_index, pool_key, alice).unwrap();
            let lower_tick_bit =
                tickmap_bit!(client, ContractRef, dex, lower_tick_index, pool_key, alice);
            let upper_tick_bit =
                tickmap_bit!(client, ContractRef, dex, upper_tick_index, pool_key, alice);
            let alice_x = balance_of!(TokenRef, client, token_x, Alice);
            let alice_y = balance_of!(TokenRef, client, token_y, Alice);
            let dex_x = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y = dex_balance!(TokenRef, client, token_y, dex);

            let zero_fee = FeeGrowth::new(0);
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
        async fn change_fee_reciever_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let (token_x, token_y) = create_tokens!(client, TokenRef, TokenRef, 500, 500);

            let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1);
            let init_tick = 0;

            let alice = ink_e2e::alice();

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let result = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );
            assert!(result.is_ok());

            let admin = ink_e2e::alice();
            let alice = address_of!(Alice);
            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            change_fee_receiver!(client, ContractRef, dex, pool_key, alice, admin);
            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            assert_eq!(pool.fee_receiver, alice);

            Ok(())
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn not_admin_change_fee_reciever_test(mut client: ink_e2e::Client<C, E>) -> () {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let (token_x, token_y) = create_tokens!(client, TokenRef, TokenRef, 500, 500);

            let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100);
            let init_tick = 0;

            let admin = ink_e2e::alice();

            create_fee_tier!(client, ContractRef, dex, fee_tier, admin);

            let result = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );
            assert!(result.is_ok());

            let user = ink_e2e::bob();
            let bob = address_of!(Bob);
            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
            change_fee_receiver!(client, ContractRef, dex, pool_key, bob, user);
        }

        #[ink_e2e::test]
        async fn remove_position_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();
            let init_tick = 0;
            let remove_position_index = 0;

            let initial_mint = 10u128.pow(10);

            let dex = create_dex!(client, ContractRef, Percentage::from_scale(1, 2));
            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_mint, initial_mint);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let lower_tick_index = -20;
            let upper_tick_index = 10;
            let liquidity_delta = Liquidity::from_integer(1_000_000);

            approve!(client, TokenRef, token_x, dex, initial_mint, alice);
            approve!(client, TokenRef, token_y, dex, initial_mint, alice);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            assert_eq!(pool_state.liquidity, liquidity_delta);

            let liquidity_delta = Liquidity::new(liquidity_delta.get() * 1_000_000);
            {
                let incorrect_lower_tick_index = lower_tick_index - 50;
                let incorrect_upper_tick_index = upper_tick_index + 50;

                approve!(client, TokenRef, token_x, dex, liquidity_delta.get(), alice);
                approve!(client, TokenRef, token_y, dex, liquidity_delta.get(), alice);

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    incorrect_lower_tick_index,
                    incorrect_upper_tick_index,
                    liquidity_delta,
                    pool_state.sqrt_price,
                    pool_state.sqrt_price,
                    alice
                );

                let position_state = get_position!(client, ContractRef, dex, 1, alice).unwrap();
                // Check position
                assert!(position_state.lower_tick_index == incorrect_lower_tick_index);
                assert!(position_state.upper_tick_index == incorrect_upper_tick_index);
            }

            let amount = 1000;
            mint!(TokenRef, client, token_x, Bob, amount);
            let amount_x = balance_of!(TokenRef, client, token_x, Bob);
            assert_eq!(amount_x, amount);

            approve!(client, TokenRef, token_x, dex, amount, bob);

            let pool_state_before =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            let swap_amount = TokenAmount::new(amount);
            let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                slippage,
                bob
            );

            let pool_state_after =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            assert_eq!(
                pool_state_after.fee_growth_global_x,
                FeeGrowth::new(49999950000049999)
            );
            assert_eq!(pool_state_after.fee_protocol_token_x, TokenAmount(1));
            assert_eq!(pool_state_after.fee_protocol_token_y, TokenAmount(0));

            assert!(pool_state_after
                .sqrt_price
                .lt(&pool_state_before.sqrt_price));

            assert_eq!(pool_state_after.liquidity, pool_state_before.liquidity);
            assert_eq!(pool_state_after.current_tick_index, -10);
            assert_ne!(pool_state_after.sqrt_price, pool_state_before.sqrt_price);

            let amount_x = balance_of!(TokenRef, client, token_x, Bob);
            let amount_y = balance_of!(TokenRef, client, token_y, Bob);
            assert_eq!(amount_x, 0);
            assert_eq!(amount_y, 993);

            // pre load dex balances
            let dex_x_before_remove = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y_before_remove = dex_balance!(TokenRef, client, token_y, dex);

            // Remove position
            let remove_result =
                remove_position!(client, ContractRef, dex, remove_position_index, alice);

            // Load states
            let position_state =
                get_position!(client, ContractRef, dex, remove_position_index, alice);
            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let lower_tick = get_tick!(client, ContractRef, dex, lower_tick_index, pool_key, alice);
            let upper_tick = get_tick!(client, ContractRef, dex, upper_tick_index, pool_key, alice);
            let lower_tick_bit =
                tickmap_bit!(client, ContractRef, dex, lower_tick_index, pool_key, alice);
            let upper_tick_bit =
                tickmap_bit!(client, ContractRef, dex, upper_tick_index, pool_key, alice);
            let alice_x = balance_of!(TokenRef, client, token_x, Alice);
            let alice_y = balance_of!(TokenRef, client, token_y, Alice);
            let dex_x = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y = dex_balance!(TokenRef, client, token_y, dex);
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
        async fn position_slippage_zero_slippage_and_inside_range(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let alice = ink_e2e::alice();
            let (dex, token_x, token_y) =
                init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
            let pool_key = create_slippage_pool_with_liquidity!(
                client,
                ContractRef,
                TokenRef,
                dex,
                token_x,
                token_y
            );

            let pool = get_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                pool_key.fee_tier
            )
            .unwrap();

            // zero slippage
            {
                let liquidity_delta = Liquidity::from_integer(1_000_000);
                let known_price = pool.sqrt_price;
                let tick = pool_key.fee_tier.tick_spacing as i32;
                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    -tick,
                    tick,
                    liquidity_delta,
                    known_price,
                    known_price,
                    alice
                );
            }
            // inside range
            {
                let liquidity_delta = Liquidity::from_integer(1_000_000);
                let known_price = SqrtPrice::new(1010000000000000000000000);
                let limit_lower = SqrtPrice::new(994734637981406576896367);
                let limit_upper = SqrtPrice::new(1025038048074314166333500);

                let tick = pool_key.fee_tier.tick_spacing as i32;

                create_position!(
                    client,
                    ContractRef,
                    dex,
                    pool_key,
                    -tick,
                    tick,
                    liquidity_delta,
                    limit_lower,
                    limit_upper,
                    alice
                );
            }

            Ok(())
        }
        #[ink_e2e::test]
        #[should_panic]
        async fn position_slippage_below_range(mut client: ink_e2e::Client<C, E>) -> () {
            let alice = ink_e2e::alice();
            let (dex, token_x, token_y) =
                init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
            let pool_key = create_slippage_pool_with_liquidity!(
                client,
                ContractRef,
                TokenRef,
                dex,
                token_x,
                token_y
            );

            let pool = get_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                pool_key.fee_tier
            )
            .unwrap();

            let liquidity_delta = Liquidity::from_integer(1_000_000);
            let known_price = SqrtPrice::new(1030000000000000000000000);
            let limit_lower = SqrtPrice::new(1014432353584998786339859);
            let limit_upper = SqrtPrice::new(1045335831204498605270797);
            let tick = pool_key.fee_tier.tick_spacing as i32;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                limit_lower,
                limit_upper,
                alice
            );
        }
        #[ink_e2e::test]
        #[should_panic]
        async fn position_slippage_above_range(mut client: ink_e2e::Client<C, E>) -> () {
            let alice = ink_e2e::alice();
            let (dex, token_x, token_y) =
                init_slippage_dex_and_tokens!(client, ContractRef, TokenRef);
            let pool_key = create_slippage_pool_with_liquidity!(
                client,
                ContractRef,
                TokenRef,
                dex,
                token_x,
                token_y
            );

            let pool = get_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                pool_key.fee_tier
            )
            .unwrap();

            let liquidity_delta = Liquidity::from_integer(1_000_000);
            let known_price = pool.sqrt_price;
            let limit_lower = SqrtPrice::new(955339206774222158009382);
            let limit_upper = SqrtPrice::new(984442481813945288458906);
            let tick = pool_key.fee_tier.tick_spacing as i32;
            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -tick,
                tick,
                liquidity_delta,
                limit_lower,
                limit_upper,
                alice
            );
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn no_liquidity_swap(mut client: ink_e2e::Client<C, E>) -> () {
            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();
            let init_tick = 0;

            let initial_mint = 10u128.pow(10);

            let dex = create_dex!(client, ContractRef, Percentage::from_scale(1, 2));
            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_mint, initial_mint);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let lower_tick_index = -10;
            let upper_tick_index = 10;

            let mint_amount = 10u128.pow(10);
            mint!(TokenRef, client, token_x, Alice, mint_amount);
            mint!(TokenRef, client, token_y, Alice, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, alice);
            approve!(client, TokenRef, token_y, dex, mint_amount, alice);

            let liquidity_delta = Liquidity::from_integer(20_006_000);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            assert_eq!(pool_state.liquidity, liquidity_delta);

            let mint_amount = 10067;
            mint!(TokenRef, client, token_x, Bob, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, bob);

            let dex_x_before = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y_before = dex_balance!(TokenRef, client, token_y, dex);

            let swap_amount = TokenAmount::new(10067);
            let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);
            let quoted_target_sqrt_price = quote!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                target_sqrt_price,
                alice
            )
            .unwrap()
            .2;

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                quoted_target_sqrt_price,
                bob
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let expected_price = calculate_sqrt_price(-10).unwrap();
            let expected_y_amount_out = 9999;

            assert_eq!(pool.liquidity, liquidity_delta);
            assert_eq!(pool.current_tick_index, lower_tick_index);
            assert_eq!(pool.sqrt_price, expected_price);

            let bob_x = balance_of!(TokenRef, client, token_x, Bob);
            let bob_y = balance_of!(TokenRef, client, token_y, Bob);
            let dex_x_after = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y_after = dex_balance!(TokenRef, client, token_y, dex);

            let delta_dex_x = dex_x_after - dex_x_before;
            let delta_dex_y = dex_y_before - dex_y_after;

            assert_eq!(bob_x, 0);
            assert_eq!(bob_y, expected_y_amount_out);
            assert_eq!(delta_dex_x, swap_amount.get());
            assert_eq!(delta_dex_y, expected_y_amount_out);
            assert_eq!(
                pool.fee_growth_global_x,
                FeeGrowth::new(29991002699190242927121)
            );
            assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
            assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(1));
            assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(0));

            let swap_amount = TokenAmount(1);
            let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);
            let quoted_target_sqrt_price = quote!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                target_sqrt_price,
                alice
            )
            .unwrap()
            .2;
            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                quoted_target_sqrt_price,
                bob
            );
        }

        #[ink_e2e::test]
        async fn liquidity_gap_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();
            let init_tick = 0;

            let initial_mint = 10u128.pow(10);

            let dex = create_dex!(client, ContractRef, Percentage::from_scale(1, 2));
            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_mint, initial_mint);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let lower_tick_index = -10;
            let upper_tick_index = 10;

            let mint_amount = 10u128.pow(10);
            mint!(TokenRef, client, token_x, Alice, mint_amount);
            mint!(TokenRef, client, token_y, Alice, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, alice);
            approve!(client, TokenRef, token_y, dex, mint_amount, alice);

            let liquidity_delta = Liquidity::from_integer(20_006_000);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            assert_eq!(pool_state.liquidity, liquidity_delta);

            let mint_amount = 10067;
            mint!(TokenRef, client, token_x, Bob, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, bob);

            let dex_x_before = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y_before = dex_balance!(TokenRef, client, token_y, dex);

            let swap_amount = TokenAmount::new(10067);
            let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);
            let quoted_target_sqrt_price = quote!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                target_sqrt_price,
                alice
            )
            .unwrap()
            .2;

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                quoted_target_sqrt_price,
                bob
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let expected_price = calculate_sqrt_price(-10).unwrap();
            let expected_y_amount_out = 9999;

            assert_eq!(pool.liquidity, liquidity_delta);
            assert_eq!(pool.current_tick_index, lower_tick_index);
            assert_eq!(pool.sqrt_price, expected_price);

            let bob_x = balance_of!(TokenRef, client, token_x, Bob);
            let bob_y = balance_of!(TokenRef, client, token_y, Bob);
            let dex_x_after = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y_after = dex_balance!(TokenRef, client, token_y, dex);

            let delta_dex_x = dex_x_after - dex_x_before;
            let delta_dex_y = dex_y_before - dex_y_after;

            assert_eq!(bob_x, 0);
            assert_eq!(bob_y, expected_y_amount_out);
            assert_eq!(delta_dex_x, swap_amount.get());
            assert_eq!(delta_dex_y, expected_y_amount_out);
            assert_eq!(
                pool.fee_growth_global_x,
                FeeGrowth::new(29991002699190242927121)
            );
            assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
            assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(1));
            assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(0));

            // Should skip gap and then swap
            let lower_tick_after_swap = -90;
            let upper_tick_after_swap = -50;
            let liquidity_delta = Liquidity::from_integer(20008000);

            approve!(client, TokenRef, token_x, dex, liquidity_delta.get(), alice);
            approve!(client, TokenRef, token_y, dex, liquidity_delta.get(), alice);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_after_swap,
                upper_tick_after_swap,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            let swap_amount = TokenAmount::new(5000);
            mint!(TokenRef, client, token_x, Bob, swap_amount.get());

            approve!(client, TokenRef, token_x, dex, swap_amount.get(), bob);

            let dex_x_before = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y_before = dex_balance!(TokenRef, client, token_y, dex);

            let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);
            let quoted_target_sqrt_price = quote!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                target_sqrt_price,
                alice
            )
            .unwrap()
            .2;

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                swap_amount,
                true,
                quoted_target_sqrt_price,
                bob
            );
            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            let bob_x = balance_of!(TokenRef, client, token_x, Bob);
            let bob_y = balance_of!(TokenRef, client, token_y, Bob);
            let dex_x_after = dex_balance!(TokenRef, client, token_x, dex);
            let dex_y_after = dex_balance!(TokenRef, client, token_y, dex);

            let delta_dex_x = dex_x_after - dex_x_before;
            let delta_dex_y = dex_y_before - dex_y_after;
            Ok(())
        }

        #[ink_e2e::test]
        async fn cross_both_side_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();
            let init_tick = 0;

            let initial_mint = 10u128.pow(10);

            let dex = create_dex!(client, ContractRef, Percentage::from_scale(1, 2));
            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_mint, initial_mint);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let lower_tick_index = -10;
            let upper_tick_index = 10;

            let mint_amount = 10u128.pow(5);
            mint!(TokenRef, client, token_x, Alice, mint_amount);
            mint!(TokenRef, client, token_y, Alice, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, alice);
            approve!(client, TokenRef, token_y, dex, mint_amount, alice);

            let liquidity_delta = Liquidity::new(20006000000000);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -20,
                lower_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            assert_eq!(pool.liquidity, liquidity_delta);

            let limit_without_cross_tick_amount = TokenAmount(10_068);
            let not_cross_amount = TokenAmount(1);
            let min_amount_to_cross_from_tick_price = TokenAmount(3);
            let crossing_amount_by_amount_out = TokenAmount(20136101434);

            let mint_amount = limit_without_cross_tick_amount.get()
                + not_cross_amount.get()
                + min_amount_to_cross_from_tick_price.get()
                + crossing_amount_by_amount_out.get();

            mint!(TokenRef, client, token_x, Alice, mint_amount);
            mint!(TokenRef, client, token_y, Alice, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, alice);
            approve!(client, TokenRef, token_y, dex, mint_amount, alice);

            let pool_before =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            let limit_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                limit_without_cross_tick_amount,
                true,
                limit_sqrt_price,
                alice
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let expected_tick = -10;
            let expected_price = calculate_sqrt_price(expected_tick).unwrap();

            assert_eq!(pool.current_tick_index, expected_tick);
            assert_eq!(pool.liquidity, pool_before.liquidity);
            assert_eq!(pool.sqrt_price, expected_price);

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                min_amount_to_cross_from_tick_price,
                true,
                limit_sqrt_price,
                alice
            );

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                false,
                min_amount_to_cross_from_tick_price,
                true,
                SqrtPrice::new(MAX_SQRT_PRICE),
                alice
            );

            let massive_x = 10u128.pow(19);
            let massive_y = 10u128.pow(19);

            mint!(TokenRef, client, token_x, Alice, massive_x);
            mint!(TokenRef, client, token_y, Alice, massive_y);
            approve!(client, TokenRef, token_x, dex, massive_x, alice);
            approve!(client, TokenRef, token_y, dex, massive_y, alice);

            let massive_liquidity_delta = Liquidity::new(19996000399699881985603000000);

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -20,
                0,
                massive_liquidity_delta,
                SqrtPrice::new(MIN_SQRT_PRICE),
                SqrtPrice::new(MAX_SQRT_PRICE),
                alice
            );

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                TokenAmount(1),
                false,
                limit_sqrt_price,
                alice
            );

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                false,
                TokenAmount(2),
                true,
                SqrtPrice::new(MAX_SQRT_PRICE),
                alice
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            assert_eq!(pool.current_tick_index, -20);
            assert_eq!(
                pool.fee_growth_global_x,
                FeeGrowth::new(29991002699190242927121)
            );
            assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
            assert_eq!(pool.fee_protocol_token_x, TokenAmount(4));
            assert_eq!(pool.fee_protocol_token_y, TokenAmount(2));
            assert_eq!(
                pool.liquidity,
                Liquidity::new(19996000399699901991603000000)
            );
            assert_eq!(pool.sqrt_price, SqrtPrice::new(999500149964999999999999));

            let final_last_tick =
                get_tick!(client, ContractRef, dex, -20, pool_key, alice).unwrap();
            assert_eq!(final_last_tick.fee_growth_outside_x, FeeGrowth::new(0));
            assert_eq!(final_last_tick.fee_growth_outside_y, FeeGrowth::new(0));
            assert_eq!(
                final_last_tick.liquidity_change,
                Liquidity::new(19996000399699901991603000000)
            );

            let final_lower_tick =
                get_tick!(client, ContractRef, dex, -10, pool_key, alice).unwrap();
            assert_eq!(
                final_lower_tick.fee_growth_outside_x,
                FeeGrowth::new(29991002699190242927121)
            );
            assert_eq!(final_lower_tick.fee_growth_outside_y, FeeGrowth::new(0));
            assert_eq!(final_lower_tick.liquidity_change, Liquidity::new(0));

            let final_upper_tick =
                get_tick!(client, ContractRef, dex, 10, pool_key, alice).unwrap();
            assert_eq!(final_upper_tick.fee_growth_outside_x, FeeGrowth::new(0));
            assert_eq!(final_upper_tick.fee_growth_outside_y, FeeGrowth::new(0));
            assert_eq!(
                final_upper_tick.liquidity_change,
                Liquidity::new(20006000000000)
            );

            Ok(())
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn cross_both_side_not_cross_case_test(mut client: ink_e2e::Client<C, E>) -> () {
            let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10);
            let alice = ink_e2e::alice();
            let bob = ink_e2e::bob();
            let init_tick = 0;

            let initial_mint = 10u128.pow(10);

            let dex = create_dex!(client, ContractRef, Percentage::from_scale(1, 2));
            let (token_x, token_y) =
                create_tokens!(client, TokenRef, TokenRef, initial_mint, initial_mint);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

            create_fee_tier!(client, ContractRef, dex, fee_tier, alice);

            let pool = create_pool!(
                client,
                ContractRef,
                dex,
                token_x,
                token_y,
                fee_tier,
                init_tick
            );

            let lower_tick_index = -10;
            let upper_tick_index = 10;

            let mint_amount = 10u128.pow(5);
            mint!(TokenRef, client, token_x, Alice, mint_amount);
            mint!(TokenRef, client, token_y, Alice, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, alice);
            approve!(client, TokenRef, token_y, dex, mint_amount, alice);

            let liquidity_delta = Liquidity::new(20006000000000);

            let pool_state =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -20,
                lower_tick_index,
                liquidity_delta,
                pool_state.sqrt_price,
                pool_state.sqrt_price,
                alice
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            assert_eq!(pool.liquidity, liquidity_delta);

            let limit_without_cross_tick_amount = TokenAmount(10_068);
            let not_cross_amount = TokenAmount(1);
            let min_amount_to_cross_from_tick_price = TokenAmount(3);
            let crossing_amount_by_amount_out = TokenAmount(20136101434);

            let mint_amount = limit_without_cross_tick_amount.get()
                + not_cross_amount.get()
                + min_amount_to_cross_from_tick_price.get()
                + crossing_amount_by_amount_out.get();

            mint!(TokenRef, client, token_x, Alice, mint_amount);
            mint!(TokenRef, client, token_y, Alice, mint_amount);

            approve!(client, TokenRef, token_x, dex, mint_amount, alice);
            approve!(client, TokenRef, token_y, dex, mint_amount, alice);

            let pool_before =
                get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();

            let limit_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                limit_without_cross_tick_amount,
                true,
                limit_sqrt_price,
                alice
            );

            let pool = get_pool!(client, ContractRef, dex, token_x, token_y, fee_tier).unwrap();
            let expected_tick = -10;
            let expected_price = calculate_sqrt_price(expected_tick).unwrap();

            assert_eq!(pool.current_tick_index, expected_tick);
            assert_eq!(pool.liquidity, pool_before.liquidity);
            assert_eq!(pool.sqrt_price, expected_price);

            let slippage = SqrtPrice::new(MIN_SQRT_PRICE);
            let target_sqrt_price = quote!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                not_cross_amount,
                true,
                slippage,
                alice
            )
            .unwrap()
            .2;

            swap!(
                client,
                ContractRef,
                dex,
                pool_key,
                true,
                not_cross_amount,
                true,
                target_sqrt_price,
                alice
            );
        }
    }
}
