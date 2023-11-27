use crate::InvariantError;
use ink::storage::Mapping;

use crate::math::types::percentage::Percentage;

#[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
// key(fee: Percentage, tick_spacing: u16)
pub struct FeeTierKey(pub Percentage, pub u16);

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct FeeTiers {
    fee_tiers: Mapping<FeeTierKey, ()>,
}

impl FeeTiers {
    pub fn get_fee_tier(&self, key: FeeTierKey) -> Result<(), InvariantError> {
        let fee_tier = self
            .fee_tiers
            .get(&key)
            .ok_or(InvariantError::FeeTierNotFound)?;
        Ok(fee_tier)
    }
    pub fn add_fee_tier(&mut self, key: FeeTierKey) -> Result<(), InvariantError> {
        if self.fee_tiers.get(&key).is_some() {
            return Err(InvariantError::FeeTierAlreadyAdded);
        }
        self.fee_tiers.insert(&key, &());
        Ok(())
    }
    pub fn remove_fee_tier(&mut self, key: FeeTierKey) -> Result<(), InvariantError> {
        self.fee_tiers
            .get(&key)
            .ok_or(InvariantError::FeeTierNotFound)?;
        self.fee_tiers.remove(&key);
        Ok(())
    }
}
