#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

extern crate alloc;
mod contracts;
pub mod math;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractErrors {
    InsufficientSenderBalance,
    InsufficientLPLocked,
    PairNotFound,
    MintFailed,
    BurnFailed,
    SwapFailed,
    NotAnAdmin,
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
    FeeTierAlreadyAdded,
    NotAFeeReceiver,
    ZeroLiquidity,
}
#[ink::contract]
pub mod contract {
    use crate::ContractErrors;
    use traceable_result::unwrap;

    use crate::contracts::state::State;
    use crate::contracts::Balances;
    use crate::contracts::FeeTierKey;
    use crate::contracts::Pool;
    use crate::contracts::Tick;
    use crate::contracts::Tickmap;
    use crate::contracts::{FeeTier, FeeTiers, PoolKey, Pools, Position, Positions, Ticks}; //
    use crate::math::check_tick;
    use crate::math::percentage::Percentage;
    use crate::math::sqrt_price::sqrt_price::SqrtPrice;
    use crate::math::token_amount::TokenAmount;
    use crate::math::types::liquidity::Liquidity;
    use crate::math::{compute_swap_step, MAX_SQRT_PRICE, MIN_SQRT_PRICE};
    use decimal::*;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;
    use openbrush::contracts::traits::psp22::PSP22Ref;

    #[derive(Debug)]
    pub struct OrderPair {
        pub x: (AccountId, Balance),
        pub y: (AccountId, Balance),
    }

    pub struct CalculateSwapResult {
        pub amount_in: TokenAmount,
        pub amount_out: TokenAmount,
        pub pool: Pool,
        pub ticks: Vec<Tick>,
    }

    #[derive(scale::Decode, Default, scale::Encode, Clone, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,)
    )]
    pub struct TokenPairs(pub Vec<(AccountId, AccountId)>);

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        balances: Balances,
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

        #[ink(message)]
        pub fn get_protocol_fee(&self) -> Percentage {
            self.state.protocol_fee
        }

        #[ink(message)]
        pub fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), ContractErrors> {
            let mut pool = self.pools.get_pool(pool_key)?;
            let caller = self.env().caller();

            if pool.fee_receiver != caller {
                return Err(ContractErrors::NotAFeeReceiver);
            }

            let (fee_protocol_token_x, fee_protocol_token_y) = pool.withdraw_protocol_fee(pool_key);
            self.pools.update_pool(pool_key, &pool);

            PSP22Ref::transfer(
                &pool_key.token_x,
                pool.fee_receiver,
                fee_protocol_token_x.get(),
                vec![],
            )
            .ok();

            PSP22Ref::transfer(
                &pool_key.token_y,
                pool.fee_receiver,
                fee_protocol_token_y.get(),
                vec![],
            )
            .ok();

            Ok(())
        }

        #[ink(message)]
        pub fn change_protocol_fee(
            &mut self,
            protocol_fee: Percentage,
        ) -> Result<(), ContractErrors> {
            if self.env().caller() != self.state.admin {
                return Err(ContractErrors::NotAnAdmin);
            }

            self.state.protocol_fee = protocol_fee;
            Ok(())
        }

        #[ink(message)]
        pub fn change_fee_receiver(
            &mut self,
            pool_key: PoolKey,
            fee_receiver: AccountId,
        ) -> Result<(), ContractErrors> {
            let caller = self.env().caller();

            if caller != self.state.admin {
                return Err(ContractErrors::NotAnAdmin);
            }

            let mut pool = self.pools.get_pool(pool_key)?;
            pool.fee_receiver = fee_receiver;
            self.pools.update_pool(pool_key, &pool);

            Ok(())
        }

        pub fn create_tick(
            &mut self,
            pool_key: PoolKey,
            index: i32,
        ) -> Result<Tick, ContractErrors> {
            check_tick(index, pool_key.fee_tier.tick_spacing)
                .map_err(|_| ContractErrors::InvalidTickIndexOrTickSpacing)?;

            let pool = self.pools.get_pool(pool_key)?;

            let tick_option = self.ticks.get_tick(pool_key, index);
            if tick_option.is_some() {
                return Err(ContractErrors::TickAlreadyExist);
            }

            let current_timestamp = self.env().block_timestamp();
            let tick = Tick::create(index, &pool, current_timestamp);
            self.ticks.add_tick(pool_key, index, tick);

            self.tickmap
                .flip(true, index, pool_key.fee_tier.tick_spacing, pool_key);

            Ok(tick)
        }

        #[ink(message)]
        pub fn create_position(
            &mut self,
            pool_key: PoolKey,
            lower_tick: i32,
            upper_tick: i32,
            liquidity_delta: Liquidity,
            slippage_limit_lower: SqrtPrice,
            slippage_limit_upper: SqrtPrice,
        ) -> Result<Position, ContractErrors> {
            // liquidity delta = 0 => return
            if liquidity_delta == Liquidity::new(0) {
                return Err(ContractErrors::ZeroLiquidity);
            }
            let mut pool = self.pools.get_pool(pool_key)?;

            let mut lower_tick = self
                .ticks
                .get_tick(pool_key, lower_tick)
                .unwrap_or(self.create_tick(pool_key, lower_tick)?);

            let mut upper_tick = self
                .ticks
                .get_tick(pool_key, upper_tick)
                .unwrap_or(self.create_tick(pool_key, upper_tick)?);

            let current_timestamp = self.env().block_timestamp();
            let current_block_number = self.env().block_number() as u64;

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
            );

            let caller = self.env().caller();
            self.positions.add(caller, position);

            self.ticks.add_tick(pool_key, lower_tick.index, lower_tick);
            self.ticks.add_tick(pool_key, upper_tick.index, upper_tick);

            PSP22Ref::transfer_from(
                &pool_key.token_x,
                self.env().caller(),
                self.env().account_id(),
                x.get(),
                vec![],
            )
            .ok();
            PSP22Ref::transfer_from(
                &pool_key.token_y,
                self.env().caller(),
                self.env().account_id(),
                y.get(),
                vec![],
            )
            .ok();

            Ok(position)
        }

        pub fn calculate_swap(
            &self,
            pool_key: PoolKey,
            x_to_y: bool,
            amount: TokenAmount,
            by_amount_in: bool,
            sqrt_price_limit: SqrtPrice,
        ) -> Result<CalculateSwapResult, ContractErrors> {
            if amount.is_zero() {
                return Err(ContractErrors::AmountIsZero);
            }

            let mut ticks: Vec<Tick> = vec![];

            let mut pool = self.pools.get_pool(pool_key)?;
            let current_timestamp = self.env().block_timestamp();

            if x_to_y {
                if pool.sqrt_price > sqrt_price_limit
                    && sqrt_price_limit <= SqrtPrice::new(MAX_SQRT_PRICE)
                {
                    return Err(ContractErrors::WrongLimit);
                }
            } else {
                if pool.sqrt_price > sqrt_price_limit
                    && sqrt_price_limit <= SqrtPrice::new(MIN_SQRT_PRICE)
                {
                    return Err(ContractErrors::WrongLimit);
                }
            }

            let mut remaining_amount = amount;

            let mut total_amount_in = TokenAmount(0);
            let mut total_amount_out = TokenAmount(0);

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

                pool.sqrt_price = result.next_sqrt_price;

                total_amount_in += result.amount_in + result.fee_amount;
                total_amount_out += result.amount_out;

                // Fail if price would go over swap limit
                if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
                    return Err(ContractErrors::PriceLimitReached);
                }

                // TODO: refactor
                let mut tick = Tick::default();

                let update_limiting_tick = limiting_tick.map(|(index, bool)| {
                    if bool {
                        tick = self.ticks.get_tick(pool_key, index).unwrap();
                        (index, Some(&mut tick))
                    } else {
                        (index, None)
                    }
                });

                pool.cross_tick(
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

                ticks.push(tick);
            }

            if total_amount_out.get() == 0 {
                return Err(ContractErrors::NoGainSwap);
            }

            Ok(CalculateSwapResult {
                amount_in: total_amount_in,
                amount_out: total_amount_out,
                pool,
                ticks,
            })
        }

        #[ink(message)]
        pub fn swap(
            &mut self,
            pool_key: PoolKey,
            x_to_y: bool,
            amount: TokenAmount,
            by_amount_in: bool,
            sqrt_price_limit: SqrtPrice,
        ) -> Result<(), ContractErrors> {
            let calculate_swap_result =
                self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

            for tick in calculate_swap_result.ticks.iter() {
                self.ticks.update_tick(pool_key, tick.index, tick);
            }

            self.pools
                .update_pool(pool_key, &calculate_swap_result.pool);

            if x_to_y {
                PSP22Ref::transfer_from(
                    &pool_key.token_x,
                    self.env().caller(),
                    self.env().account_id(),
                    calculate_swap_result.amount_in.get(),
                    vec![],
                )
                .ok();
                PSP22Ref::transfer(
                    &pool_key.token_y,
                    self.env().caller(),
                    calculate_swap_result.amount_out.get(),
                    vec![],
                )
                .ok();
            } else {
                PSP22Ref::transfer_from(
                    &pool_key.token_y,
                    self.env().caller(),
                    self.env().account_id(),
                    calculate_swap_result.amount_in.get(),
                    vec![],
                )
                .ok();
                PSP22Ref::transfer(
                    &pool_key.token_x,
                    self.env().caller(),
                    calculate_swap_result.amount_out.get(),
                    vec![],
                )
                .ok();
            };

            Ok(())
        }

        #[ink(message)]
        pub fn quote(
            &self,
            pool_key: PoolKey,
            x_to_y: bool,
            amount: TokenAmount,
            by_amount_in: bool,
            sqrt_price_limit: SqrtPrice,
        ) -> Result<(TokenAmount, TokenAmount, SqrtPrice), ContractErrors> {
            let calculate_swap_result =
                self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

            Ok((
                calculate_swap_result.amount_in,
                calculate_swap_result.amount_out,
                calculate_swap_result.pool.sqrt_price,
            ))
        }

        #[ink(message)]
        pub fn transfer_position(
            &mut self,
            index: u32,
            receiver: AccountId,
        ) -> Result<(), ContractErrors> {
            let caller = self.env().caller();
            self.positions.transfer(caller, index, receiver)?;

            Ok(())
        }

        // positions list features
        // #[ink(message)]
        // pub fn add_position(&mut self) {
        //     let caller = self.env().caller();
        //     self.positions.add(caller, Position::default());
        // }

        #[ink(message)]
        pub fn get_position(&mut self, index: u32) -> Option<Position> {
            let caller = self.env().caller();
            self.positions.get(caller, index)
        }

        #[ink(message)]
        pub fn get_all_positions(&mut self) -> Vec<Position> {
            let caller = self.env().caller();
            self.positions.get_all(caller)
        }

        #[ink(message)]
        pub fn update_position_seconds_per_liquidity(
            &mut self,
            index: u32,
            pool_key: PoolKey,
        ) -> Result<(), ContractErrors> {
            let caller = self.env().caller();

            let mut position = self
                .positions
                .get(caller, index)
                .ok_or(ContractErrors::PositionNotFound)?;

            let current_timestamp = self.env().block_number();

            let lower_tick = self
                .ticks
                .get_tick(pool_key, position.lower_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let upper_tick = self
                .ticks
                .get_tick(pool_key, position.upper_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let pool = self.pools.get_pool(pool_key)?;

            position.update_seconds_per_liquidity(
                pool,
                lower_tick,
                upper_tick,
                current_timestamp as u64,
            );
            Ok(())
        }

        #[ink(message)]
        pub fn position_claim_fee(
            &mut self,
            index: u32,
            pool_key: PoolKey,
        ) -> Result<(TokenAmount, TokenAmount), ContractErrors> {
            let caller = self.env().caller();
            let contract = self.env().account_id();
            let current_timestamp = self.env().block_number();

            let mut position = self
                .positions
                .get(caller, index)
                .ok_or(ContractErrors::PositionNotFound)?;

            let lower_tick = self
                .ticks
                .get_tick(pool_key, position.lower_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let upper_tick = self
                .ticks
                .get_tick(pool_key, position.upper_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let pool = self.pools.get_pool(pool_key)?;

            let (token_x, token_y) = position.claim_fee(
                pool,
                upper_tick,
                lower_tick,
                current_timestamp as u64,
                pool_key,
                contract,
                caller,
            );
            Ok((token_x, token_y))
        }

        #[ink(message)]
        pub fn remove_position(
            &mut self,
            index: u32,
        ) -> Result<(TokenAmount, TokenAmount), ContractErrors> {
            let caller = self.env().caller();
            let current_timestamp = self.env().block_number();

            let mut position = self
                .positions
                .get(caller, index)
                .ok_or(ContractErrors::PositionNotFound)?;

            let lower_tick = &mut self
                .ticks
                .get_tick(position.pool_key, position.lower_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let upper_tick = &mut self
                .ticks
                .get_tick(position.pool_key, position.upper_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let pool = &mut self.pools.get_pool(position.pool_key)?;

            let (amount_x, amount_y, deinitialize_lower_tick, deinitialize_upper_tick) = position
                .remove(
                    pool,
                    current_timestamp as u64,
                    lower_tick,
                    upper_tick,
                    position.pool_key.fee_tier.tick_spacing,
                );
            if deinitialize_lower_tick {
                self.tickmap.flip(
                    false,
                    lower_tick.index,
                    position.pool_key.fee_tier.tick_spacing,
                    position.pool_key,
                );
            }
            if deinitialize_upper_tick {
                self.tickmap.flip(
                    false,
                    upper_tick.index,
                    position.pool_key.fee_tier.tick_spacing,
                    position.pool_key,
                );
            }
            self.positions.remove(caller, index).unwrap();

            PSP22Ref::transfer_from(
                &position.pool_key.token_x,
                self.env().account_id(),
                self.env().caller(),
                amount_x.get(),
                vec![],
            )
            .ok();
            PSP22Ref::transfer_from(
                &position.pool_key.token_y,
                self.env().account_id(),
                self.env().caller(),
                amount_y.get(),
                vec![],
            )
            .ok();

            Ok((amount_x, amount_y))
        }

        // Fee tiers
        #[ink(message)]
        pub fn add_fee_tier(
            &mut self,
            fee: Percentage,
            tick_spacing: u16,
        ) -> Result<(), ContractErrors> {
            if self.env().caller() != self.state.admin {
                return Err(ContractErrors::NotAnAdmin);
            }

            if tick_spacing == 0 {
                return Err(ContractErrors::InvalidTickSpacing);
            }
            let fee_tier_key = FeeTierKey(fee, tick_spacing);

            if self.fee_tiers.get_fee_tier(fee_tier_key).is_some() {
                return Err(ContractErrors::FeeTierAlreadyAdded);
            } else {
                self.fee_tiers.add_fee_tier(fee_tier_key);
                self.fee_tier_keys.push(fee_tier_key);
                Ok(())
            }
        }

        #[ink(message)]
        pub fn get_fee_tier(&self, key: FeeTierKey) -> Option<()> {
            self.fee_tiers.get_fee_tier(key)
        }

        #[ink(message)]
        pub fn remove_fee_tier(&mut self, key: FeeTierKey) {
            self.fee_tiers.remove_fee_tier(key);
            self.fee_tier_keys.retain(|&x| x != key);
        }

        // Pools
        #[ink(message)]
        pub fn add_pool(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            fee_tier: FeeTier,
            init_tick: i32,
        ) -> Result<(), ContractErrors> {
            let current_timestamp = self.env().block_timestamp();
            let key = PoolKey::new(token_0, token_1, fee_tier);
            self.pool_keys.push(key);
            self.pools
                .add_pool(key, current_timestamp, self.state.admin, init_tick)
        }

        #[ink(message)]
        pub fn get_pool(
            &self,
            token_0: AccountId,
            token_1: AccountId,
            fee_tier: FeeTier,
        ) -> Result<Pool, ContractErrors> {
            let key: PoolKey = PoolKey::new(token_0, token_1, fee_tier);
            self.pools.get_pool(key)
        }

        fn remove_pool(&mut self, key: PoolKey) {
            self.pools.remove_pool(key);
            self.pool_keys.retain(|&x| x != key);
        }

        // Ticks
        fn add_tick(&mut self, key: PoolKey, index: i32, tick: Tick) {
            self.ticks.add_tick(key, index, tick);
        }

        fn get_tick(&self, key: PoolKey, index: i32) -> Option<Tick> {
            self.ticks.get_tick(key, index)
        }
        fn get_tickmap_bit(&self, key: PoolKey, index: i32) -> bool {
            self.tickmap.get(index, key.fee_tier.tick_spacing, key)
        }
        fn remove_tick(&mut self, key: PoolKey, index: i32) {
            self.ticks.remove_tick(key, index);
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
            let result = contract.add_pool(
                token_0,
                token_1,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
                0,
            );
            assert!(result.is_ok());
            let result = contract.add_pool(
                token_1,
                token_0,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
                0,
            );
            assert_eq!(result, Err(ContractErrors::PoolAlreadyExist));
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
            assert_eq!(result, Err(ContractErrors::PoolNotFound));
            let result = contract.add_pool(
                token_0,
                token_1,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
                0,
            );
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
            let pool_key = PoolKey::new(
                token_0,
                token_1,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 2,
                },
            );
            let result = contract.create_tick(pool_key, MAX_TICK + 1);
            assert_eq!(result, Err(ContractErrors::InvalidTickIndexOrTickSpacing));
            let result = contract.create_tick(pool_key, 1);
            assert_eq!(result, Err(ContractErrors::InvalidTickIndexOrTickSpacing));
            let result = contract.create_tick(pool_key, 0);
            assert_eq!(result, Err(ContractErrors::PoolNotFound));
            let _ = contract.add_pool(pool_key.token_x, pool_key.token_y, pool_key.fee_tier, 0);
            let result = contract.create_tick(pool_key, 0);
            assert!(result.is_ok());
            let result = contract.create_tick(pool_key, 0);
            assert_eq!(result, Err(ContractErrors::TickAlreadyExist));
        }

        #[ink::test]
        fn test_fee_tiers() {
            let mut contract = Contract::new(Percentage::new(0));
            let fee_tier_key = FeeTierKey(Percentage::new(1), 10u16);
            let fee_tier_value = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 10u16,
            };

            contract.add_fee_tier(Percentage::new(1), 10u16).unwrap();
            assert_eq!(contract.fee_tier_keys.len(), 1);
            contract
                .add_fee_tier(Percentage::new(1), 10u16)
                .unwrap_err();
            contract.remove_fee_tier(fee_tier_key);
            assert_eq!(contract.fee_tier_keys.len(), 0);
        }

        #[ink::test]
        fn test_ticks() {
            let mut contract = Contract::new(Percentage::new(0));
            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 50u16,
            };
            let pool_key = PoolKey {
                token_x: AccountId::from([0x0; 32]),
                token_y: AccountId::from([0x0; 32]),
                fee_tier,
            };
            let tick = Tick::default();
            let index = 10i32;
            contract.add_tick(pool_key, index, tick);
            let recieved_tick = contract.get_tick(pool_key, index);
            assert_eq!(Some(tick), recieved_tick);
            contract.remove_tick(pool_key, index);
            let recieved_tick = contract.get_tick(pool_key, index);
            assert_eq!(None, recieved_tick);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    pub mod e2e_tests {
        use ink::prelude::vec;
        use ink::prelude::vec::Vec;
        use ink_e2e::build_message;
        use openbrush::contracts::psp22::psp22_external::PSP22;
        use openbrush::traits::Balance;
        use test_helpers::{
            address_of, approve, balance_of, create_dex, create_fee_tier, create_pool,
            create_position, create_standard_fee_tiers, create_tokens, dex_balance,
            get_all_positions, get_fee_tier, get_pool, get_position, remove_position,
        };
        use token::TokenRef;

        use super::*;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn constructor_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500);
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

            let fee_tier = FeeTier {
                fee: Percentage::new(0),
                tick_spacing: 1,
            };
            let pool = create_pool!(client, ContractRef, dex, token_x, token_y, fee_tier, 10);

            approve!(client, TokenRef, token_x, dex, 500);
            approve!(client, TokenRef, token_y, dex, 500);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier);
            let alice = ink_e2e::alice();

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
        async fn test_positions(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            let (token_x, token_y) = create_tokens!(client, TokenRef, TokenRef, 500, 500);

            let alice = ink_e2e::alice();

            let fee_tier = FeeTier {
                fee: Percentage::new(0),
                tick_spacing: 1,
            };
            let pool = create_pool!(client, ContractRef, dex, token_x, token_y, fee_tier, 10);

            approve!(client, TokenRef, token_x, dex, 50);
            approve!(client, TokenRef, token_y, dex, 50);

            let pool_key = PoolKey::new(token_x, token_y, fee_tier);

            // Get all Alice positions - should be empty
            let alice_positions = get_all_positions!(client, ContractRef, dex, alice);

            assert_eq!(alice_positions, vec![]);

            // // Alice adds 3 positions

            let first_position = create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -1,
                1,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let second_position = create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -2,
                2,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            let third_position = create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -3,
                3,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                alice
            )
            .unwrap();

            // // Get all Alice positions
            let alice_positions = get_all_positions!(client, ContractRef, dex, alice);
            assert_eq!(alice_positions.len(), 3);

            // // Bob adds 2 positions
            let bob = ink_e2e::bob();
            let first_position = create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -4,
                4,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                bob
            )
            .unwrap();

            let second_position = create_position!(
                client,
                ContractRef,
                dex,
                pool_key,
                -5,
                5,
                Liquidity::new(10),
                SqrtPrice::new(0),
                SqrtPrice::max_instance(),
                bob
            )
            .unwrap();

            // // Get all Bob positions
            let bob_positions = get_all_positions!(client, ContractRef, dex, bob);
            assert_eq!(bob_positions.len(), 2);

            let alice_second_position = get_position!(client, ContractRef, dex, 1, alice);
            assert!(alice_second_position.is_some());

            let bob_first_position = get_position!(client, ContractRef, dex, 0, bob);
            assert!(bob_first_position.is_some());

            remove_position!(client, ContractRef, dex, 2, alice);

            let alice_positions = get_all_positions!(client, ContractRef, dex, alice);
            // println!("Alice positions = {:?}", alice_positions);

            let alice_third_position = get_position!(client, ContractRef, dex, 2, alice);

            // Bob tries to remove position out of range
            remove_position!(client, ContractRef, dex, 9999, bob);
            let bob_positions = get_all_positions!(client, ContractRef, dex, bob);
            assert_eq!(bob_positions.len(), 2);

            // Bob removes first position
            remove_position!(client, ContractRef, dex, 1, bob);
            // Get all Bob positions
            let bob_positions = get_all_positions!(client, ContractRef, dex, bob);
            assert_eq!(bob_positions.len(), 1);
            Ok(())
        }

        #[ink_e2e::test]
        async fn create_fee_tier_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let dex = create_dex!(client, ContractRef, Percentage::new(0));
            create_fee_tier!(client, ContractRef, dex, Percentage::new(0), 10u16);
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

            let fee_tier = FeeTier {
                fee: Percentage::from_scale(5, 1),
                tick_spacing: 100,
            };
            let init_tick = 0;

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
    }
}
