use decimal::*;

use crate::alloc::string::ToString;

#[decimal(28)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct DecimalExampleX {
    pub v: u128,
}
