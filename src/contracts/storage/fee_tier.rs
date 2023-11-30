use crate::{math::types::percentage::Percentage, InvariantError};
use decimal::*;
#[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}

impl FeeTier {
    pub fn new(fee: Percentage, tick_spacing: u16) -> Result<Self, InvariantError> {
        if tick_spacing == 0 || tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee > Percentage::from_integer(1) {
            return Err(InvariantError::InvalidFee);
        }

        Ok(Self { fee, tick_spacing })
    }
}
