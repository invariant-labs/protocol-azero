use crate::alloc::string::ToString;
use decimal::*;

#[decimal(0)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scale::Decode, scale::Encode)]

pub struct TokenAmount(pub u128);
