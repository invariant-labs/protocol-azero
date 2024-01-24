#![cfg_attr(not(feature = "std"), no_std, no_main)]
// #![no_std]
extern crate alloc;

pub mod clamm;
pub mod consts;
pub mod log;
pub mod types;

pub use clamm::*;
pub use consts::*;
pub use log::*;
pub use types::*;

#[cfg(not(feature = "wasm"))]
#[ink::contract]
#[cfg(not(feature = "wasm"))]
pub mod contract {
    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}
