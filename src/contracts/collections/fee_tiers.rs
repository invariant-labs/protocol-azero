use crate::{math::types::percentage::Percentage, InvariantError};
use ink::storage::Mapping;

#[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct FeeTierKey(pub Percentage, pub u16);

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct FeeTiers {
    fee_tiers: Mapping<FeeTierKey, ()>,
}

impl FeeTiers {
    pub fn add(&mut self, fee_tier_key: &FeeTierKey) -> Result<(), InvariantError> {
        if self.fee_tiers.get(&fee_tier_key).is_some() {
            return Err(InvariantError::FeeTierAlreadyExist);
        }

        self.fee_tiers.insert(&fee_tier_key, &());
        Ok(())
    }

    pub fn remove(&mut self, fee_tier_key: &FeeTierKey) -> Result<(), InvariantError> {
        self.fee_tiers
            .get(fee_tier_key)
            .ok_or(InvariantError::FeeTierNotFound)?;

        self.fee_tiers.remove(&fee_tier_key);
        Ok(())
    }

    pub fn get(&self, fee_tier_key: &FeeTierKey) -> Option<()> {
        self.fee_tiers.get(fee_tier_key)
    }
}
