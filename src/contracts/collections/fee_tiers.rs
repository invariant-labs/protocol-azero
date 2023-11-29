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
    pub fn add(&mut self, fee_tier_key: FeeTierKey) -> Result<(), InvariantError> {
        self.fee_tiers
            .get(fee_tier_key)
            .map_or(Ok(()), |_| Err(InvariantError::FeeTierAlreadyExist))?;

        self.fee_tiers.insert(fee_tier_key, &());
        Ok(())
    }

    pub fn remove(&mut self, fee_tier_key: FeeTierKey) -> Result<(), InvariantError> {
        self.fee_tiers
            .get(fee_tier_key)
            .ok_or(InvariantError::FeeTierNotFound)?;

        self.fee_tiers.remove(fee_tier_key);
        Ok(())
    }

    pub fn get(&self, fee_tier_key: FeeTierKey) -> Option<()> {
        self.fee_tiers.get(fee_tier_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::percentage::Percentage;
    use decimal::*;

    #[ink::test]
    fn test_add() {
        let fee_tiers = &mut FeeTiers::default();
        let fee_tier_key = FeeTierKey(Percentage::new(0), 1);
        let new_fee_tier_key = FeeTierKey(Percentage::new(0), 2);

        fee_tiers.add(fee_tier_key).unwrap();
        assert_eq!(fee_tiers.get(fee_tier_key), Some(()));
        assert_eq!(fee_tiers.get(new_fee_tier_key), None);

        let result = fee_tiers.add(fee_tier_key);
        assert_eq!(result, Err(InvariantError::FeeTierAlreadyExist));
    }

    #[ink::test]
    fn test_remove() {
        let fee_tiers = &mut FeeTiers::default();
        let fee_tier_key = FeeTierKey(Percentage::new(0), 1);

        fee_tiers.add(fee_tier_key).unwrap();

        fee_tiers.remove(fee_tier_key).unwrap();
        assert_eq!(fee_tiers.get(fee_tier_key), None);

        let result = fee_tiers.remove(fee_tier_key);
        assert_eq!(result, Err(InvariantError::FeeTierNotFound));
    }
}
