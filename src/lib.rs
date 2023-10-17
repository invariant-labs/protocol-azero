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
}

#[ink::contract]
pub mod contract {
    use crate::contracts::logic::traits::pair::Pair;
    use crate::contracts::pair::pair::PairField;
    use crate::contracts::State;
    use crate::math::percentage::Percentage;
    use crate::ContractErrors;
    use decimal::*;
    use ink::prelude::{vec, vec::Vec};
    use ink::storage::Mapping;
    use openbrush::contracts::traits::psp22::PSP22Ref;

    #[derive(Debug)]
    pub struct OrderPair {
        pub x: (AccountId, Balance),
        pub y: (AccountId, Balance),
    }

    #[ink::storage_item]
    #[derive(Debug)]
    pub struct Pairs {
        pairs: Mapping<(AccountId, AccountId), PairField>,
        token_pairs: TokenPairs,
        length: u64,
    }

    #[derive(scale::Decode, Default, scale::Encode, Clone, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,)
    )]
    pub struct TokenPairs(Vec<(AccountId, AccountId)>);

    impl Default for Pairs {
        fn default() -> Self {
            Pairs {
                pairs: Default::default(),
                length: Default::default(),
                token_pairs: Default::default(),
            }
        }
    }

    impl Pairs {
        fn _create_pair(&mut self, ordered_pair: OrderPair) -> PairField {
            assert_ne!(ordered_pair.x.0, ordered_pair.y.0);
            let pair = self.pairs.get((&ordered_pair.x.0, &ordered_pair.y.0));
            match pair {
                Some(pair) => pair,
                None => {
                    let pair_field = PairField {
                        token_x: ordered_pair.x.0,
                        token_y: ordered_pair.y.0,
                        ..Default::default()
                    };
                    self.pairs
                        .insert((ordered_pair.x.0, ordered_pair.y.0), &pair_field);
                    self.length += 1;
                    self.token_pairs
                        .0
                        .push((ordered_pair.x.0, ordered_pair.y.0));
                    pair_field
                }
            }
        }
        fn _update_pair(&mut self, pair: PairField) {
            self.pairs.insert((pair.token_x, pair.token_y), &pair);
        }

        fn _get_pair(&self, ordered_pair: &OrderPair) -> Option<PairField> {
            self.pairs.get((ordered_pair.x.0, ordered_pair.y.0))
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

    #[ink::storage_item]
    #[derive(Debug)]
    pub struct Positions {
        positions: Mapping<AccountId, (u32, Vec<Position>)>,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Position {
        value: u8,
        id: u8,
    }

    impl scale::EncodeLike<Vec<Position>> for Position {}

    impl Default for Positions {
        fn default() -> Positions {
            Positions {
                positions: Default::default(),
            }
        }
    }

    impl Positions {
        pub fn add_position(&mut self, caller: AccountId) {
            let positions_length = &self.get_length(caller);
            match positions_length {
                Some(_x) => {
                    let mut current_positions = self.positions.get(caller).unwrap();
                    let next_index = (current_positions.0 + 1) as u8;
                    current_positions.0 += 1;
                    current_positions.1.push(Position {
                        value: next_index * 11,
                        id: next_index,
                    });
                    self.positions.insert(caller, &current_positions);
                }
                None => {
                    let new_position = (0, vec![Position { value: 0, id: 0 }]);
                    self.positions.insert(caller, &new_position);
                }
            }
        }
        pub fn remove_position(&mut self, caller: AccountId, index: u32) {
            let positions_length = self.get_length(caller);

            if let Some(x) = positions_length {
                if x >= index {
                    let mut current_positions = self.positions.get(caller).unwrap();
                    current_positions.0 -= 1;
                    current_positions.1.remove(index as usize);
                    self.positions.insert(caller, &current_positions);
                }
            }
        }

        pub fn get_all_positions(&self, caller: AccountId) -> Vec<Position> {
            self.positions.get(&caller).unwrap_or_default().1
        }

        pub fn get_position(&mut self, caller: AccountId, index: u32) -> Option<Position> {
            let positions_length = self.get_length(caller);
            match positions_length {
                Some(x) => {
                    if index <= x {
                        let positions = self.positions.get(caller).unwrap().1;
                        positions.get(index as usize).cloned()
                    } else {
                        None
                    }
                }
                None => None,
            }
        }
        fn get_length(&mut self, caller: AccountId) -> Option<u32> {
            let positions = self.positions.get(&caller);
            match positions {
                Some(x) => Some(x.0),
                None => None,
            }
        }
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        pairs: Pairs,
        // (x, y, user), balance
        balances: Mapping<(AccountId, AccountId, AccountId), Balance>,
        positions: Positions,
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
                            .get((pair.token_x, pair.token_y, caller))
                            .unwrap_or_default();
                        balance += lp;
                        self.balances
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
    }

    #[cfg(test)]
    mod tests {
        use decimal::*;

        use super::*;

        #[ink::test]
        fn initialize_works() {
            let _ = Contract::new(Percentage::new(0));
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
                        Position { value: 0, id: 0 },
                        Position { value: 11, id: 1 },
                        Position { value: 22, id: 2 },
                        Position { value: 33, id: 3 },
                        Position { value: 44, id: 4 }
                    ],
                    positions
                )
            }
            // basic position operations
            {
                let position = contract.get_position(2).unwrap();
                assert_eq!(Position { value: 22, id: 2 }, position);

                contract.remove_position(2);

                let position = contract.get_position(2).unwrap();
                assert_eq!(Position { value: 33, id: 3 }, position);
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
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    pub mod e2e_tests {

        // use super::*;
        // use ink_e2e::build_message;
        // use ink_e2e::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        use openbrush::contracts::psp22::psp22_external::PSP22;
        use test_helpers::address_of;
        use token::TokenRef;

        #[ink_e2e::test]
        async fn constructor_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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
                    Position { value: 0, id: 0 },
                    Position { value: 11, id: 1 },
                    Position { value: 22, id: 2 },
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
                vec![Position { value: 0, id: 0 }, Position { value: 11, id: 1 },]
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

            assert_eq!(Position { value: 11, id: 1 }, alice_second_positions);

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

            assert_eq!(Position { value: 0, id: 0 }, bob_first_position);

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

            assert_eq!(Position { value: 22, id: 2 }, alice_second_positions);

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

            assert_eq!(Position { value: 11, id: 1 }, bob_first_position);

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
