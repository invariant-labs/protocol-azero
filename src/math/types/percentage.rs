use core::convert::{TryFrom, TryInto};
use decimal::*;
#[decimal(12)]
#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    // scale_info::TypeInfo,
    // ink::storage::traits::StorageLayout,
)]
#[cfg_attr(
    feature = "std",
    derive(
        scale_info::TypeInfo,
        scale::Decode,
        scale::Encode,
        ink::storage::traits::StorageLayout
    )
)]
pub struct Percentage {
    pub v: u64,
}
