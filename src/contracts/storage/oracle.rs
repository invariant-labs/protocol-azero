use crate::math::types::sqrt_price::SqrtPrice;
use ink::prelude::{vec, vec::Vec};

#[derive(Default, Debug, PartialEq, Clone, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Oracle {
    pub data: Vec<Record>,
    pub head: u16,
    pub amount: u16,
    pub size: u16,
}

#[derive(Default, Debug, PartialEq, Copy, Clone, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Record {
    pub timestamp: u64,
    pub price: SqrtPrice,
}

impl Oracle {
    pub fn new() -> Self {
        Self {
            data: vec![Record::default(); 256],
            head: 0,
            amount: 0,
            size: 256,
        }
    }
}
