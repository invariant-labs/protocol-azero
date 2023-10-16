#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub mod contracts;

#[ink::contract]
mod flipper {

    #[ink(storage)]
    pub struct Flipper {
        value: u128,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { value: 0 }
        }

        #[ink(message)]
        pub fn get(&self) -> u128 {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let flipper = Flipper::new();
            assert_eq!(flipper.get(), 0);
        }

        #[cfg(all(test, feature = "e2e-tests"))]
        mod e2e_tests {
            /// Imports all the definitions from the outer scope so we can use them here.
            use super::*;
    
            /// A helper function used for calling contract messages.
            use ink_e2e::build_message;
    
            /// The End-to-End test `Result` type.
            type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    
            /// We test that we can upload and instantiate the contract using its default constructor.
            #[ink_e2e::test]
            async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
                // Given
                let constructor = FlipperRef::new();
    
                // When
                let contract_account_id = client
                    .instantiate("invariant_protocol", &ink_e2e::alice(), constructor, 0, None)
                    .await
                    .expect("instantiate failed")
                    .account_id;
    
                // Then
                let get = build_message::<FlipperRef>(contract_account_id.clone())
                    .call(|flipper| flipper.get());
                let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
                assert!(matches!(get_result.return_value(), 0));
    
                Ok(())
            }
        }
    }
}
