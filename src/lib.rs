#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
mod contracts;
pub mod math;
pub mod tests;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum InvariantError {
    NotAdmin,
    NotFeeReceiver,
    PoolAlreadyExist,
    PoolNotFound,
    TickAlreadyExist,
    InvalidTickIndexOrTickSpacing,
    PositionNotFound,
    TickNotFound,
    FeeTierNotFound,
    PoolKeyNotFound,
    AmountIsZero,
    WrongLimit,
    PriceLimitReached,
    NoGainSwap,
    InvalidTickSpacing,
    FeeTierAlreadyExist,
    PoolKeyAlreadyExist,
    UnauthorizedFeeReceiver,
    ZeroLiquidity,
    TransferError,
    TokensAreTheSame,
    AmountUnderMinimumAmountOut,
    InvalidFee,
}
#[ink::contract]
pub mod contract {
    use crate::InvariantError;
    // use math::fee_growth::FeeGrowth;
    use crate::contracts::state::State;
    use crate::contracts::Invariant;
    use crate::contracts::Pool;
    use crate::contracts::PoolKeys;
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
    #[derive(Default, Debug, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct QuoteResult {
        pub amount_in: TokenAmount,
        pub amount_out: TokenAmount,
        pub target_sqrt_price: SqrtPrice,
        pub ticks: Vec<Tick>,
    }

    #[derive(scale::Decode, Default, scale::Encode, Clone, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,)
    )]
    pub struct Hop {
        pub pool_key: PoolKey,
        pub x_to_y: bool,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        positions: Positions,
        pools: Pools,
        tickmap: Tickmap,
        ticks: Ticks,
        fee_tiers: FeeTiers,
        pool_keys: PoolKeys,
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
            } else if pool.sqrt_price >= sqrt_price_limit
                || sqrt_price_limit < SqrtPrice::new(MIN_SQRT_PRICE)
            {
                return Err(InvariantError::WrongLimit);
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

                unwrap!(pool.add_fee(result.fee_amount, x_to_y, self.state.protocol_fee));
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
                    self.emit_cross_tick_event(caller, pool_key, limiting_tick.unwrap().0);

                    ticks.push(tick);
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

        fn remove_tick(&mut self, key: PoolKey, index: i32) -> Result<(), InvariantError> {
            self.ticks.remove(key, index)?;

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
                return Err(InvariantError::NotFeeReceiver);
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
                return Err(InvariantError::NotAdmin);
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
                self.ticks.update(pool_key, tick.index, tick)?;
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
                    current_timestamp,
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

        #[ink(message)]
        fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.state.admin {
                return Err(InvariantError::NotAdmin);
            }

            self.fee_tiers.add(fee_tier)?;

            Ok(())
        }

        #[ink(message)]
        fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
            let caller = self.env().caller();

            if caller != self.state.admin {
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
            init_tick: i32,
        ) -> Result<(), InvariantError> {
            let current_timestamp = self.env().block_timestamp();

            if !self.fee_tiers.contains(fee_tier) {
                return Err(InvariantError::FeeTierNotFound);
            };

            let pool_key = PoolKey::new(token_0, token_1, fee_tier)?;
            if self.pools.get(pool_key).is_ok() {
                return Err(InvariantError::PoolAlreadyExist);
            };
            let pool = Pool::create(init_tick, current_timestamp, self.state.admin);
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
        fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
            self.ticks.get(key, index)
        }

        #[ink(message)]
        fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool {
            self.tickmap.get(index, key.fee_tier.tick_spacing, key)
        }

        #[ink(message)]
        fn get_pools(&self) -> Vec<PoolKey> {
            self.pool_keys.get_all()
        }

        #[ink(message)]
        fn get_fee_tiers(&self) -> Vec<FeeTier> {
            self.fee_tiers.get_all()
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
