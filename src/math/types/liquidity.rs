#[allow(unused_imports)]
use crate::alloc::string::ToString;
use core::convert::{TryFrom, TryInto};
use decimal::*;
#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use tsify::Tsify;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[decimal(6)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[cfg_attr(not(feature = "wasm"), derive(scale::Encode, scale::Decode))]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
#[cfg_attr(
    feature = "wasm",
    derive(Serialize, Deserialize, Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Liquidity(#[cfg_attr(feature = "wasm", tsify(type = "bigint"))] pub u128);
