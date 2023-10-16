#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub mod contracts;

#[ink::contract]
mod contract {
    use math::token_amount::TokenAmount;

    use crate::contracts::{Pool, Tickmap};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Flipper {
        value: u128,
        tickmap: Tickmap,
        pool: Pool,
        token_amount: TokenAmount,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
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
    }
}
