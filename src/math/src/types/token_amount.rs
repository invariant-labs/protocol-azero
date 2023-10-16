#[allow(unused_imports)]
use crate::alloc::string::ToString;
use decimal::*;

#[decimal(0)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct TokenAmount(pub u128);
