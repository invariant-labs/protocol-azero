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
    scale::Decode,
    scale::Encode,
    scale_info::TypeInfo,
    ink::storage::traits::StorageLayout,
)]
// #[cfg_attr(feature = "std", derive())]
pub struct Percentage {
    pub v: u64,
}
