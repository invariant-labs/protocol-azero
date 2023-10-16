#[allow(unused_imports)]
use crate::alloc::string::ToString;
use decimal::*;

#[decimal(6)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Liquidity {
    pub v: u128,
}
