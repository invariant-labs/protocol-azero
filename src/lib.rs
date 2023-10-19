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
}
#[ink::contract]
pub mod contract {
    use crate::{
        contracts::logic::traits::pair::Pair,
        contracts::pair::pair::PairField,
        // contracts::storage::{balances::Balances, fee_tiers::FeeTierKey, tick::Tick, Pairs},
        ContractErrors,
    };

    use traceable_result::unwrap;

    use crate::contracts::state::State;
    use crate::contracts::Balances;
    use crate::contracts::FeeTierKey;
    use crate::contracts::Pairs;
    use crate::contracts::Pool;
    use crate::contracts::Tick;
    use crate::contracts::Tickmap;
    use crate::contracts::{FeeTier, FeeTiers, PoolKey, Pools, Position, Positions, Ticks}; //
    use crate::math::check_tick;
    use crate::math::percentage::Percentage;
    use crate::math::sqrt_price::sqrt_price::SqrtPrice;
    use crate::math::token_amount::TokenAmount;
    use crate::math::types::liquidity::Liquidity;
    use crate::math::MAX_TICK;
    use decimal::*;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;
    use openbrush::contracts::traits::psp22::PSP22Ref;

    #[derive(Debug)]
    pub struct OrderPair {
        pub x: (AccountId, Balance),
        pub y: (AccountId, Balance),
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
        pairs: Pairs,
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
            let mut pool = self.pools.get_pool(pool_key)?;

            let mut lower_tick = match self.ticks.get_tick(pool_key, lower_tick) {
                Some(tick) => tick,
                None => self.create_tick(pool_key, lower_tick)?,
            };

            let mut upper_tick = match self.ticks.get_tick(pool_key, upper_tick) {
                Some(tick) => tick,
                None => self.create_tick(pool_key, upper_tick)?,
            };

            let current_timestamp = self.env().block_timestamp();
            let current_block_number = self.env().block_number() as u64;

            let (position, x, y) = Position::create(
                &mut pool,
                &mut lower_tick,
                &mut upper_tick,
                current_timestamp,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
                current_block_number,
                pool_key.fee_tier.tick_spacing,
            );

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

        #[ink(message)]
        pub fn create_pair(&mut self, token_0: AccountId, token_1: AccountId) -> PairField {
            let ordered_pair = self.pairs._order_tokens(token_0, token_1, 0, 0);
            self.pairs._create_pair(ordered_pair)
        }

        #[ink(message)]
        pub fn mint_to(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            amount_0: Balance,
            amount_1: Balance,
        ) -> Result<(), ContractErrors> {
            let ordered_pair = self
                .pairs
                ._order_tokens(token_0, token_1, amount_0, amount_1);

            let caller = self.env().caller();

            let mut pair = self
                .get_pair(&ordered_pair)
                .ok_or(ContractErrors::PairNotFound)?;

            let sender_balance_x = PSP22Ref::balance_of(&ordered_pair.x.0, caller);
            let sender_balance_y = PSP22Ref::balance_of(&ordered_pair.y.0, caller);
            if sender_balance_x < ordered_pair.x.1 && sender_balance_y < ordered_pair.y.1 {
                return Err(ContractErrors::InsufficientSenderBalance);
            } else {
                let contract = Self::env().account_id();
                let result = pair._mint(caller, contract, ordered_pair);
                match result {
                    Ok(lp) => {
                        let mut balance = self
                            .balances
                            .v
                            .get((pair.token_x, pair.token_y, caller))
                            .unwrap_or_default();
                        balance += lp;
                        self.balances
                            .v
                            .insert((pair.token_x, pair.token_y, caller), &balance);
                        self.pairs._update_pair(pair);
                        Ok(())
                    }
                    _ => {
                        return Err(ContractErrors::MintFailed);
                    }
                }
            }
        }

        #[ink(message)]
        pub fn burn_from(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            amount: Balance,
        ) -> Result<(), ContractErrors> {
            let ordered_pair = self.pairs._order_tokens(token_0, token_1, amount, amount);
            let caller = self.env().caller();

            let balance = self
                .balances
                .v
                .get((ordered_pair.x.0, ordered_pair.y.0, caller))
                .ok_or(ContractErrors::InsufficientLPLocked)?;

            if balance < amount {
                return Err(ContractErrors::InsufficientLPLocked);
            }

            let mut pair = self
                .get_pair(&ordered_pair)
                .ok_or(ContractErrors::PairNotFound)?;

            let contract = self.env().account_id();
            let result = pair._burn(caller, contract, amount);

            match result {
                Ok(_) => {
                    let new_lp = balance - amount;
                    self.balances
                        .v
                        .insert((pair.token_x, pair.token_y, caller), &new_lp);
                    self.pairs._update_pair(pair);
                    Ok(())
                }
                _ => {
                    return Err(ContractErrors::BurnFailed);
                }
            }
        }

        #[ink(message)]
        pub fn swap_tokens(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            amount: Balance,
            in_token_0: bool,
        ) -> Result<(), ContractErrors> {
            let ordered_pair = self.pairs._order_tokens(token_0, token_1, amount, amount);
            let caller = self.env().caller();

            let mut pair = self
                .get_pair(&ordered_pair)
                .ok_or(ContractErrors::PairNotFound)?;

            let contract = self.env().account_id();
            let result = pair._swap(contract, caller, amount, in_token_0);

            match result {
                Ok(_) => {
                    self.pairs._update_pair(pair);
                    Ok(())
                }
                _ => return Err(ContractErrors::SwapFailed),
            }
        }

        pub fn get_pair(&self, ordered_pair: &OrderPair) -> Option<PairField> {
            self.pairs._get_pair(ordered_pair)
        }

        // Factory features
        #[ink(message)]
        pub fn all_pairs(&self) -> TokenPairs {
            self.pairs.token_pairs.clone()
        }

        #[ink(message)]
        pub fn all_pairs_length(&mut self) -> u64 {
            self.pairs.length
        }

        // positions list features
        #[ink(message)]
        pub fn add_position(&mut self) {
            let caller = self.env().caller();
            self.positions.add_position(caller)
        }

        #[ink(message)]
        pub fn get_position(&mut self, index: u32) -> Option<Position> {
            let caller = self.env().caller();
            self.positions.get_position(caller, index)
        }
        #[ink(message)]
        pub fn get_all_positions(&mut self) -> Vec<Position> {
            let caller = self.env().caller();
            self.positions.get_all_positions(caller)
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
                .get_position(caller, index)
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
                .get_position(caller, index)
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
            pool_key: PoolKey,
        ) -> Result<(TokenAmount, TokenAmount), ContractErrors> {
            let caller = self.env().caller();
            let current_timestamp = self.env().block_number();

            let mut position = self
                .positions
                .get_position(caller, index)
                .ok_or(ContractErrors::PositionNotFound)?;

            let lower_tick = &mut self
                .ticks
                .get_tick(pool_key, position.lower_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let upper_tick = &mut self
                .ticks
                .get_tick(pool_key, position.upper_tick_index)
                .ok_or(ContractErrors::TickNotFound)?;

            let pool = &mut self.pools.get_pool(pool_key)?;

            let (amount_x, amount_y, deinitialize_lower_tick, deinitialize_upper_tick) = position
                .remove(
                    pool,
                    current_timestamp as u64,
                    lower_tick,
                    upper_tick,
                    pool_key.fee_tier.tick_spacing,
                );
            if deinitialize_lower_tick {
                self.tickmap.flip(
                    false,
                    lower_tick.index,
                    pool_key.fee_tier.tick_spacing,
                    pool_key,
                );
            }
            if deinitialize_upper_tick {
                self.tickmap.flip(
                    false,
                    lower_tick.index,
                    pool_key.fee_tier.tick_spacing,
                    pool_key,
                );
            }
            self.positions.remove_position(caller, index);
            Ok((amount_x, amount_y))
        }

        // Fee tiers
        #[ink(message)]
        pub fn add_fee_tier(&mut self, key: FeeTierKey, value: FeeTier) {
            self.fee_tiers.add_fee_tier(key, value);
            self.fee_tier_keys.push(key);
        }

        #[ink(message)]
        pub fn get_fee_tier(&self, key: FeeTierKey) -> Option<FeeTier> {
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
        fn remove_tick(&mut self, key: PoolKey, index: i32) {
            self.ticks.remove_tick(key, index);
        }
    }

    #[cfg(test)]
    mod tests {

        use super::*;

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
        fn create_new_pairs() {
            let mut contract = Contract::new(Percentage::new(0));
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
            let pair = contract.create_pair(token_0, token_1);
            assert_eq!(token_0, pair.token_x);
            assert_eq!(token_1, pair.token_y);
        }

        #[ink::test]
        fn test_mapping_length() {
            let mut contract = Contract::new(Percentage::new(0));
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
            let pair = contract.create_pair(token_0, token_1);
            assert_eq!(token_0, pair.token_x);
            assert_eq!(token_1, pair.token_y);
            let len = contract.all_pairs_length();
            assert_eq!(len, 1);
        }
        #[ink::test]
        fn test_positions() {
            let mut contract = Contract::new(Percentage::new(0));
            contract.add_position();
            contract.add_position();
            contract.add_position();
            contract.add_position();
            contract.add_position();
            // get all positions
            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 50u16,
            };
            let pool_key = PoolKey {
                token_x: AccountId::from([0x01; 32]),
                token_y: AccountId::from([0x02; 32]),
                fee_tier,
            };

            {
                let positions = contract.get_all_positions();
                assert_eq!(
                    vec![
                        Position {
                            ..Default::default()
                        },
                        Position {
                            ..Default::default()
                        },
                        Position {
                            ..Default::default()
                        },
                        Position {
                            ..Default::default()
                        },
                        Position {
                            ..Default::default()
                        }
                    ],
                    positions
                )
            }
            // basic position operations
            {
                let position = contract.get_position(2).unwrap();
                assert_eq!(
                    Position {
                        ..Default::default()
                    },
                    position
                );

                contract.remove_position(2, pool_key);

                let position = contract.get_position(2).unwrap();
                assert_eq!(
                    Position {
                        ..Default::default()
                    },
                    position
                );
            }
            // get positions out of range
            {
                let position_out_of_range = contract.get_position(99);
                assert_eq!(position_out_of_range, None)
            }
            // remove positions out of range
            {
                contract.remove_position(99, pool_key);
            }
        }

        #[ink::test]
        fn test_fee_tiers() {
            let mut contract = Contract::new(Percentage::new(0));
            let fee_tier_key = FeeTierKey(Percentage::new(1), 10u16);
            let fee_tier_value = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 50u16,
            };

            contract.add_fee_tier(fee_tier_key, fee_tier_value);
            assert_eq!(contract.fee_tier_keys.len(), 1);

            let recieved_fee_tier = contract.get_fee_tier(fee_tier_key);
            assert_eq!(Some(fee_tier_value), recieved_fee_tier);

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
        use test_helpers::address_of;
        use token::TokenRef;

        use super::*;
        // use crate::token::TokenRef;

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
            let constructor = ContractRef::new(Percentage::new(0));
            let contract: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

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
            let constructor = ContractRef::new(Percentage::new(0));
            let contract: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

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
            let constructor = TokenRef::new(500);
            let token_x: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let constructor = TokenRef::new(500);
            let token_y: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));
            let contract_address: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let fee_tier = FeeTier {
                fee: Percentage::new(0),
                tick_spacing: 1,
            };

            {
                let _msg = build_message::<ContractRef>(contract_address.clone())
                    .call(|contract| contract.add_pool(token_x, token_y, fee_tier, 0));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("add pool failed")
            };

            {
                let _msg = build_message::<TokenRef>(token_x.clone())
                    .call(|sc| sc.increase_allowance(contract_address.clone(), 500));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("increaase allowance failed")
            };
            {
                let _msg = build_message::<TokenRef>(token_y.clone())
                    .call(|sc| sc.increase_allowance(contract_address.clone(), 500));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("increaase allowance failed")
            };

            let pool_key = PoolKey::new(token_x, token_y, fee_tier);

            {
                let _msg =
                    build_message::<ContractRef>(contract_address.clone()).call(|contract| {
                        contract.create_position(
                            pool_key,
                            -10,
                            10,
                            Liquidity::new(10),
                            SqrtPrice::new(0),
                            SqrtPrice::max_instance(),
                        )
                    });
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("create position failed")
            };

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_positions(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 50u16,
            };
            let pool_key = PoolKey {
                token_x: AccountId::from([0x01; 32]),
                token_y: AccountId::from([0x02; 32]),
                fee_tier,
            };

            let constructor = ContractRef::new(Percentage::new(0));
            let contract: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            // Get all Alice positions - should be empty
            let alice_positions = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_all_positions());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("getting posisitons failed")
            }
            .return_value();
            assert_eq!(alice_positions, vec![]);

            // Alice is adding 3 positions
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.add_position());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("added position failed")
            };
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.add_position());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("added position failed")
            };
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.add_position());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("added position failed")
            };
            // Get all Alice positions
            let alice_positions = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_all_positions());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("getting posisitons failed")
            }
            .return_value();
            assert_eq!(
                alice_positions,
                vec![
                    Position {
                        ..Default::default()
                    },
                    Position {
                        ..Default::default()
                    },
                    Position {
                        ..Default::default()
                    },
                ]
            );

            // Bob adds 2 positions
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.add_position());
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("added position failed")
            };
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.add_position());
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("added position failed")
            };

            // Get all Alice positions
            let bob_positions = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_all_positions());
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("getting posisitons failed")
            }
            .return_value();
            assert_eq!(
                bob_positions,
                vec![
                    Position {
                        ..Default::default()
                    },
                    Position {
                        ..Default::default()
                    },
                ]
            );

            let alice_second_positions = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_position(1));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Position recieving failed")
            }
            .return_value()
            .unwrap();

            assert_eq!(
                Position {
                    ..Default::default()
                },
                alice_second_positions
            );

            let bob_first_position = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_position(0));
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("Position recieving failed")
            }
            .return_value()
            .unwrap();

            assert_eq!(
                Position {
                    ..Default::default()
                },
                bob_first_position
            );

            // Alice removes second position
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.remove_position(1, pool_key));
                client.call(&ink_e2e::alice(), _msg, 0, None).await;
            }

            let alice_second_positions = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_position(1));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Position recieving failed")
            }
            .return_value()
            .unwrap();

            assert_eq!(
                Position {
                    ..Default::default()
                },
                alice_second_positions
            );

            // Bob tires to remove position out of range
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.remove_position(99, pool_key));
                client.call(&ink_e2e::bob(), _msg, 0, None).await;
            }

            // Bob removes position first position
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.remove_position(0, pool_key));
                client.call(&ink_e2e::bob(), _msg, 0, None).await;
            }
            // Bob's first position
            let bob_first_position = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.get_position(0));
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("Position recieving failed")
            }
            .return_value()
            .unwrap();

            assert_eq!(
                Position {
                    ..Default::default()
                },
                bob_first_position
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn create_pair_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500);
            let token_x: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = TokenRef::new(500);
            let token_y: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));
            let contract: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.create_pair(token_x, token_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed")
            };
            Ok(())
        }

        #[ink_e2e::test]
        async fn create_xy_and_yx(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500);
            let token_x: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = TokenRef::new(500);
            let token_y: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));
            let contract: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let xy = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.create_pair(token_x, token_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed")
            }
            .return_value();
            let yx = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.create_pair(token_y, token_x));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed")
            }
            .return_value();

            let all_pairs_length = {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.all_pairs_length());
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed")
            }
            .return_value();

            assert_eq!(all_pairs_length, 1);
            assert_eq!(xy.token_x, yx.token_x);
            assert_eq!(xy.token_y, yx.token_y);
            Ok(())
        }

        #[ink_e2e::test]
        async fn token_mint_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500);
            let token_x: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = TokenRef::new(500);
            let token_y: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));
            let dex: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            // Pair x/y created
            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.create_pair(token_x, token_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed");
            };

            let amount_x = Balance::from(50u128);
            let amount_y = Balance::from(25u128);

            // approve token x
            let _result = {
                let _msg = build_message::<TokenRef>(token_x.clone())
                    .call(|sc| sc.increase_allowance(dex.clone(), 100));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };
            // approve token y
            let _result = {
                let _msg = build_message::<TokenRef>(token_y.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), 100));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };

            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.mint_to(token_x, token_y, amount_x, amount_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Mint failed");
            };

            // Verify if users and contract has correct amount of tokens
            {
                let alice_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let alice_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(450, alice_token_x);
                assert_eq!(475, alice_token_y);

                let dex_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let dex_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(50, dex_token_x);
                assert_eq!(25, dex_token_y);
            }
            Ok(())
        }

        #[ink_e2e::test]
        async fn single_token_mint_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500);
            let token_x: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = TokenRef::new(500);
            let token_y: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));
            let dex: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            // Pair x/y created
            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.create_pair(token_x, token_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed");
            };

            let amount_x = Balance::from(50u128);
            let amount_y = Balance::from(0u128);

            // approve token x
            let _result = {
                let _msg = build_message::<TokenRef>(token_x.clone())
                    .call(|sc| sc.increase_allowance(dex.clone(), 100));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };
            // approve token y
            let _result = {
                let _msg = build_message::<TokenRef>(token_y.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), 100));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };

            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.mint_to(token_x, token_y, amount_x, amount_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Mint failed");
            };

            // Verify if users and contract has correct amount of tokens
            {
                let alice_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let alice_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(450, alice_token_x);
                assert_eq!(500, alice_token_y);

                let dex_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let dex_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(50, dex_token_x);
                assert_eq!(0, dex_token_y);
            }
            Ok(())
        }

        #[ink_e2e::test]
        async fn token_burn_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500);
            let token_x: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = TokenRef::new(500);
            let token_y: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));
            let dex: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            // Pair x/y created
            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.create_pair(token_x, token_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed");
            };

            let amount_x = Balance::from(50u128);
            let amount_y = Balance::from(50u128);

            // approve token x
            let _result = {
                let _msg = build_message::<TokenRef>(token_x.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), amount_x));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };
            // approve token y
            let _result = {
                let _msg = build_message::<TokenRef>(token_y.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), amount_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };

            // mint some
            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.mint_to(token_x, token_y, amount_x, amount_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Mint failed");
            };

            let lp_amount = Balance::from(10u128);

            // approve token x
            let _result = {
                let _msg = build_message::<TokenRef>(token_x.clone())
                    .call(|sc| sc.increase_allowance(dex.clone(), lp_amount));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };
            // approve token y
            let _result = {
                let _msg = build_message::<TokenRef>(token_y.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), lp_amount));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };
            // burn one
            let result = {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.burn_from(token_x, token_y, lp_amount));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("burn failed");
            };

            // Verify if users and contract has correct amount of tokens
            {
                let alice_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let alice_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(450, alice_token_x);
                assert_eq!(450, alice_token_y);

                let dex_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let dex_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(50, dex_token_x);
                assert_eq!(50, dex_token_y);
            }

            Ok(())
        }

        #[ink_e2e::test]
        async fn token_swap_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(500);
            let token_x: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = TokenRef::new(500);
            let token_y: AccountId = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let constructor = ContractRef::new(Percentage::new(0));
            let dex: AccountId = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            // Pair x/y created
            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.create_pair(token_x, token_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Create pair failed");
            };

            // approve token x
            let amount_x = Balance::from(50u128);
            let amount_y = Balance::from(50u128);

            // approve token x
            let _result = {
                let _msg = build_message::<TokenRef>(token_x.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), amount_x));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };
            // approve token y
            let _result = {
                let _msg = build_message::<TokenRef>(token_y.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), amount_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };

            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.mint_to(token_x, token_y, amount_x, amount_y));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Mint failed");
            };

            // Check if contract recieved minted tokens
            {
                let dex_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let dex_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(dex_token_x, amount_x);
                assert_eq!(dex_token_y, amount_y);
            }

            let amount_to_swap = Balance::from(10u128);

            // approve token x
            let _result = {
                let _msg = build_message::<TokenRef>(token_x.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), amount_to_swap));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };
            // approve token y
            let _result = {
                let _msg = build_message::<TokenRef>(token_y.clone())
                    .call(|contract| contract.increase_allowance(dex.clone(), amount_to_swap));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };

            // swap x to y
            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.swap_tokens(token_x, token_y, amount_to_swap, true));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Swap Failed");
            };

            {
                let _msg = build_message::<ContractRef>(dex.clone())
                    .call(|contract| contract.swap_tokens(token_x, token_y, amount_to_swap, false));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Swap Failed");
            };

            // Verify if users and contract has correct amount of tokens
            {
                let alice_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let alice_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(address_of!(Alice)));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(450, alice_token_x);
                assert_eq!(450, alice_token_y);

                let dex_token_x = {
                    let _msg = build_message::<TokenRef>(token_x.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();
                let dex_token_y = {
                    let _msg = build_message::<TokenRef>(token_y.clone())
                        .call(|contract| contract.balance_of(dex.clone()));
                    client
                        .call(&ink_e2e::alice(), _msg, 0, None)
                        .await
                        .expect("Balance of failed")
                }
                .return_value();

                assert_eq!(50, dex_token_x);
                assert_eq!(50, dex_token_y);
            }
            Ok(())
        }
    }
}
