use crate::alloc::string::ToString;
use decimal::*;

#[decimal(12)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Percentage {
    pub v: u64,
}
