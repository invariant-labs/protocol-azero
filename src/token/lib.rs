#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]
// pub use my_psp22::*;
pub use openbrush::traits::{AccountId, Storage};

pub use self::token::TokenRef;

/// Most basic PSP22 token.W
/// // #[openbrush::implementation(Ownable, PSP22, PSP22Mintable)]
#[openbrush::implementation(Ownable, PSP22)]
#[openbrush::contract]
#[allow(clippy::let_unit_value)] // Clippy-speciefic workaround for errors
pub mod token {
    use core::result::Result;

    // use ink::prelude::vec;
    use openbrush::contracts::psp22::{psp22, PSP22Error, PSP22Impl};

    use crate::*;

    /*use openbrush::{
        contracts::{
            ownable::*,
            psp22::{self, extensions::mintable, psp22::Internal, Data, PSP22Error},
        },
        modifiers,
        traits::Storage,
    };*/

    #[ink(storage)]
    #[derive(Default, Storage)]
    // if u change contract name then change it in test_helpers crate too in order to run e2e tests
    pub struct Token {
        #[storage_field]
        psp22: psp22::Data,
        // storage contract owner
        #[storage_field]
        ownable: ownable::Data,
    }

    // impl psp22::PSP22 for PspExample {}

    /// Result type alias
    // pub type Result<T> = core::result::Result<T, PSP22Error>;

    // Ownable
    // impl Ownable for PspExample {}

    impl Token {
        /// Instantiate the contract with `total_supply` tokens of supply.
        ///
        /// All the created tokens will be minted to the caller.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut instance = Self::default();

            // added ownable
            ownable::Internal::_init_with_owner(&mut instance, Self::env().caller());
            psp22::Internal::_mint_to(&mut instance, Self::env().caller(), total_supply)
                .expect("Should mint");
            // instance
            //     .psp22
            //     ._mint_to(Self::env().caller(), total_supply)
            //     .expect("Should mint");

            instance
        }

        #[ink(message)]
        // #[modifiers(only_owner)]
        pub fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            psp22::Internal::_mint_to(self, account, amount)
            // self._mint_to(account, amount)
        }

        #[ink(message)]
        // #[modifiers(only_owner)]
        pub fn transfer(
            &mut self,
            to: AccountId,
            amount: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            psp22::PSP22Impl::transfer(self, to, amount, data)
        }

        #[ink(message)]
        // #[modifiers(only_owner)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            psp22::PSP22Impl::transfer_from(self, from, to, amount, data)
        }

        // impl mintable::PSP22Mintable for PspExample {
        //     #[ink(message)]
        //     #[modifiers(only_owner)]
        //     /// Mints the `amount` of underlying tokens to the recipient identified by the `account` address.
        //     fn mint(&mut self, account: AccountId, amount: Balance) -> Result<()> {
        //         self._mint_to(account, amount)
        //     }
        // }

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> Balance {
            psp22::PSP22Impl::balance_of(self, account)
        }
    }

    #[cfg(test)]
    pub mod tests {
        use ink::primitives::AccountId;

        use super::*;

        #[ink::test]
        fn constructor_test() {
            let token = Token::new(200);
            //println!("{:?}", token);
            assert_eq!(token.psp22.supply.get().unwrap(), 200);
        }

        #[ink::test]
        fn mint_test() {
            let mut token = Token::new(200);
            // let address = AccountId::from([0x1; 32]);
            let _ = token.mint(token.ownable.owner.get().unwrap().unwrap(), 200);
            assert_eq!(token.psp22.supply.get().unwrap(), 400);
        }

        #[ink::test]
        fn transfer_test() {
            let mut token = Token::new(200);
            let sender = token.ownable.owner.get().unwrap().unwrap();
            let recipient = AccountId::from([0x7; 32]);
            let result = token.transfer(recipient, 20, vec![]).unwrap();
            let sender_balance = token.balance_of(sender);
            let recipient_balance = token.balance_of(recipient);
            assert_eq!(result, ());
            assert_eq!(sender_balance, 180);
            assert_eq!(recipient_balance, 20);
        }

        #[ink::test]
        #[should_panic]
        fn transfer_insufficient_funds_test() {
            let mut token = Token::new(200);
            let recipient = AccountId::from([0x7; 32]);
            let _ = token.transfer(recipient, 200000, vec![]).unwrap();
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    pub mod e2e_tests {

        use ink_e2e::build_message;
        use openbrush::contracts::psp22::psp22_external::PSP22;
        use test_helpers::{address_of, balance_of};

        use super::*;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn transfer_bob_alice(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(100);
            let address = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            {
                let _msg = build_message::<TokenRef>(address.clone())
                    .call(|contract| contract.mint(address_of!(Bob), 50));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Mint failed!")
            };
            {
                let _msg = build_message::<TokenRef>(address.clone())
                    .call(|contract| contract.transfer(address_of!(Alice), 50, vec![]));

                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("Trasnfer failed!");
            };

            let balance_of_alice = balance_of!(TokenRef, client, address, Alice);
            let balance_of_bob = balance_of!(TokenRef, client, address, Bob);

            assert_eq!(150, balance_of_alice);
            assert_eq!(0, balance_of_bob);

            Ok(())
        }

        #[ink_e2e::test]
        async fn mints_balance_on_new(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TokenRef::new(100);
            let address = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let result = {
                let _msg = build_message::<TokenRef>(address.clone())
                    .call(|contract| contract.balance_of(address_of!(Alice)));
                client.call_dry_run(&ink_e2e::alice(), &_msg, 0, None).await
            };

            assert!(matches!(result.return_value(), 100));

            Ok(())
        }

        #[ink_e2e::test]
        async fn transfer_adds_amount_to_destination_account(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = TokenRef::new(100);
            let address = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let result = {
                let _msg = build_message::<TokenRef>(address.clone())
                    .call(|contract| contract.transfer(address_of!(Bob), 50, vec![]));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("transfer failed")
            };

            assert!(matches!(result.return_value(), Ok(())));

            let balance_of_alice = balance_of!(TokenRef, client, address, Alice);

            let balance_of_bob = balance_of!(TokenRef, client, address, Bob);

            assert_eq!(balance_of_bob, 50, "Bob should have 50 tokens");
            assert_eq!(balance_of_alice, 50, "Alice should have 50 tokens");
            Ok(())
        }

        #[ink_e2e::test]
        #[should_panic]
        async fn insufficient_allowance_test(mut client: ink_e2e::Client<C, E>) -> () {
            let constructor = TokenRef::new(100);
            let address = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let _result = {
                let _msg = build_message::<TokenRef>(address.clone()).call(|contract| {
                    contract.transfer_from(address_of!(Alice), address_of!(Bob), 100, vec![])
                });
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("Transfer from failed")
            };
        }

        #[ink_e2e::test]
        async fn sufficient_allowance_test(mut client: ink_e2e::Client<C, E>) -> () {
            let constructor = TokenRef::new(100);
            let address = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("Instantiate failed")
                .account_id;

            let _result = {
                let _msg = build_message::<TokenRef>(address.clone())
                    .call(|contract| contract.approve(address_of!(Bob), 100));
                client
                    .call(&ink_e2e::alice(), _msg, 0, None)
                    .await
                    .expect("Approval failed")
            };

            let _result = {
                let _msg = build_message::<TokenRef>(address.clone()).call(|contract| {
                    contract.transfer_from(address_of!(Alice), address_of!(Bob), 100, vec![])
                });
                client
                    .call(&ink_e2e::bob(), _msg, 0, None)
                    .await
                    .expect("Transfer from failed")
            };

            let balance_of_alice = balance_of!(TokenRef, client, address, Alice);
            let balance_of_bob = balance_of!(TokenRef, client, address, Bob);

            assert_eq!(balance_of_bob, 100, "Bob should have 100 tokens");
            assert_eq!(balance_of_alice, 0, "Alice should have 0 tokens");
        }
    }
}
