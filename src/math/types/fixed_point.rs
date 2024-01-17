use crate::alloc::string::ToString;
use core::convert::{TryFrom, TryInto};
use decimal::*;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
#[decimal(12)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[cfg_attr(
    feature = "std",
    derive(
        scale_info::TypeInfo,
        scale::Decode,
        scale::Encode,
        ink::storage::traits::StorageLayout
    )
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FixedPoint {
    pub v: u128,
}
