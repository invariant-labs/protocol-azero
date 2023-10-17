#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]
extern crate alloc;
use decimal::*;

#[decimal(12)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct DecimalExample {
    pub v: u128,
}

#[ink::contract]
mod flipper {
    use math::liquidity::Liquidity;

    use super::*;

    #[ink(storage)]
    pub struct Flipper {
        value: u128,
        example: DecimalExample,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new() -> Self {
            let l = Liquidity::new(1);
            Self {
                value: l.get(),
                example: DecimalExample::new(1),
            }
        }

        #[ink(message)]
        pub fn get(&self) -> DecimalExample {
            self.example
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let flipper = Flipper::new();
            assert_eq!(flipper.get(), DecimalExample::new(1));
        }

        #[cfg(all(test, feature = "e2e-tests"))]
        mod e2e_tests {
            /// A helper function used for calling contract messages.
            use ink_e2e::build_message;

            /// Imports all the definitions from the outer scope so we can use them here.
            use super::*;

            /// The End-to-End test `Result` type.
            type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

            /// We test that we can upload and instantiate the contract using its default constructor.
            #[ink_e2e::test]
            async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
                // // Given
                // let constructor = FlipperRef::new();

                // // When
                // let contract_account_id = client
                //     .instantiate(
                //         "invariant_protocol",
                //         &ink_e2e::alice(),
                //         constructor,
                //         0,
                //         None,
                //     )
                //     .await
                //     .expect("instantiate failed")
                //     .account_id;

                // // Then
                // let get = build_message::<FlipperRef>(contract_account_id.clone())
                //     .call(|flipper| flipper.get());
                // let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
                // assert_eq!(get_result.return_value(), DecimalExample::new(0)); // should be 1?

                Ok(())
            }
        }
    }
}
