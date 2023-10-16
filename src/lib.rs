#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub mod contracts;

#[ink::contract]
mod contract {

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
    }
}
