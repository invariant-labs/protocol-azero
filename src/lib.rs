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
}
#[ink::contract]
pub mod contract {
    // modifiers,

    use crate::{
        contracts::logic::traits::pair::Pair,
        contracts::pair::pair::PairField,
        contracts::storage::{
            balances::Balances,
            fee_tiers::FeeTierKey,
            //        pool::Pool,
            tick::Tick,
            Pairs, // tickmap::Tickmap,
        },
        ContractErrors,
    };

    use crate::contracts::Pool;
    use crate::contracts::State;
    use crate::contracts::{FeeTier, FeeTiers, PoolKey, Position, Positions, Ticks}; // Pools
    use crate::math::percentage::Percentage;
    use decimal::*;
    use ink::prelude::{vec, vec::Vec};
    use ink::storage::Mapping;
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
        pools: Mapping<PoolKey, Pool>,
        pairs: Pairs,
        balances: Balances,
        positions: Positions,
        fee_tiers: FeeTiers,
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

        #[ink(message)]
        pub fn create_pool(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            fee_tier: FeeTier,
        ) -> Result<(), ContractErrors> {
            let pool_key = PoolKey::new(token_0, token_1, fee_tier);

            let pool_option = self.pools.get(pool_key);

            if pool_option.is_some() {
                return Err(ContractErrors::PoolAlreadyExist);
            }

            self.pools.insert(pool_key, &Pool::create(pool_key));

            Ok(())
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
        pub fn remove_position(&mut self, index: u32) {
            let caller = self.env().caller();
            self.positions.remove_position(caller, index)
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

        // // Pools
        // fn add_pool(&mut self, key: PoolKey, pool: Pool, tickmap: Tickmap) {
        //     self.pools.add_pool(key, pool, tickmap);
        //     self.pool_keys.push(key);
        // }

        // fn get_pool(&self, key: PoolKey) -> Option<(Pool, Tickmap)> {
        //     self.pools.get_pool(key)
        // }

        // fn remove_pool(&mut self, key: PoolKey) {
        //     self.pools.remove_pool(key);
        //     self.pool_keys.retain(|&x| x != key);
        // }

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
        use decimal::*;

        use super::*;
        use decimal::*;

        use crate::math::percentage::Percentage;

        #[ink::test]
        fn initialize_works() {
            let _ = Contract::new(Percentage::new(0));
        }

        #[ink::test]
        fn create_pool() {
            let mut contract = Contract::new(Percentage::new(0));
            let token_0 = AccountId::from([0x01; 32]);
            let token_1 = AccountId::from([0x02; 32]);
            let result = contract.create_pool(
                token_0,
                token_1,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
            );
            assert_eq!(result, Ok(()));
            let result = contract.create_pool(
                token_1,
                token_0,
                FeeTier {
                    fee: Percentage::new(1),
                    tick_spacing: 1,
                },
            );
            assert_eq!(result, Err(ContractErrors::PoolAlreadyExist));
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

                contract.remove_position(2);

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
                contract.remove_position(99);
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

        // #[ink::test]
        // fn test_pools() {
        //     let mut contract = Contract::new();
        //     let fee_tier = FeeTier {
        //         fee: Percentage::new(1),
        //         tick_spacing: 50u16,
        //     };
        //     let pool_key = PoolKey(
        //         AccountId::from([0x0; 32]),
        //         AccountId::from([0x0; 32]),
        //         fee_tier,
        //     );
        //     let pool = Pool::default();
        //     let tickmap = Tickmap::default();
        //     contract.add_pool(pool_key, pool, tickmap);
        //     assert_eq!(contract.pool_keys.len(), 1);

        //     let recieved_pool = contract.get_pool(pool_key);
        //     assert_eq!(Some((pool, tickmap)), recieved_pool);

        //     contract.remove_pool(pool_key);
        //     assert_eq!(contract.pool_keys.len(), 0);
        // }
        #[ink::test]
        fn test_ticks() {
            let mut contract = Contract::new(Percentage::new(0));
            let fee_tier = FeeTier {
                fee: Percentage::new(1),
                tick_spacing: 50u16,
            };
            let pool_key = PoolKey(
                AccountId::from([0x0; 32]),
                AccountId::from([0x0; 32]),
                fee_tier,
            );
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
        async fn test_positions(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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
                    .call(|contract| contract.remove_position(1));
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
                    .call(|contract| contract.remove_position(99));
                client.call(&ink_e2e::bob(), _msg, 0, None).await;
            }

            // Bob removes position first position
            {
                let _msg = build_message::<ContractRef>(contract.clone())
                    .call(|contract| contract.remove_position(0));
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
