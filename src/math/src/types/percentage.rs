use crate::alloc::string::ToString;
use decimal::*;

#[decimal(12)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scale::Decode, scale::Encode)]
pub struct Percentage {
    pub v: u64,
}
