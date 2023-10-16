use decimal::*;

use crate::alloc::string::ToString;

#[decimal(6)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scale::Decode, scale::Encode)]
pub struct Liquidity {
    pub v: u128,
}
