use ink::storage::Mapping;

use crate::contracts::FeeTier;
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
    pub fn get_fee_tier(&self, key: FeeTierKey) -> Option<()> {
        self.fee_tiers.get(&key)
    }
    pub fn add_fee_tier(&mut self, key: FeeTierKey) {
        self.fee_tiers.insert(&key, &());
    }
    pub fn remove_fee_tier(&mut self, key: FeeTierKey) {
        self.fee_tiers.remove(&key);
    }
}
