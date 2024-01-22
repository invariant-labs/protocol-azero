#[cfg(feature = "wasm")]
use crate::alloc::string::ToString;
use core::convert::{TryFrom, TryInto};
use decimal::*;

#[decimal(6)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(
        scale_info::TypeInfo,
        // scale::Decode,
        // scale::Encode,
        ink::storage::traits::StorageLayout
    )
)]
#[cfg_attr(
    feature = "wasm",
    derive(serde::Serialize, serde::Deserialize, tsify::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Liquidity {
    pub v: u128,
}
