use crate::{math::percentage::Percentage, InvariantError};
use decimal::*;

#[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct FeeTierKey(pub Percentage, pub u16);

impl Default for FeeTierKey {
    fn default() -> Self {
        Self(Percentage::new(0), 1)
    }
}

impl FeeTierKey {
    pub fn new(fee: Percentage, tick_spacing: u16) -> Result<Self, InvariantError> {
        if tick_spacing == 0 || tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee > Percentage::from_integer(1) {
            return Err(InvariantError::InvalidFee);
        }

        Ok(Self(fee, tick_spacing))
    }
}
