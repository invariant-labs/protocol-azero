use core::convert::{TryFrom, TryInto};
use decimal::*;
#[decimal(6)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[cfg_attr(
    feature = "std",
    derive(
        scale_info::TypeInfo,
        scale::Decode,
        scale::Encode,
        ink::storage::traits::StorageLayout
    )
)]
pub struct Liquidity {
    pub v: u128,
}
