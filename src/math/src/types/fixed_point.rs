use crate::alloc::string::ToString;

use decimal::*;

#[decimal(12)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct FixedPoint {
    pub v: u128,
}
