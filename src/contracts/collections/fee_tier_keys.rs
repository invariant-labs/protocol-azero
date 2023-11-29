use crate::{contracts::FeeTierKey, InvariantError};
use alloc::vec::Vec;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct FeeTierKeys {
    fee_tier_keys: Vec<FeeTierKey>,
}

impl FeeTierKeys {
    pub fn add(&mut self, fee_tier_key: FeeTierKey) -> Result<(), InvariantError> {
        if self.contains(fee_tier_key) {
            return Err(InvariantError::FeeTierKeyAlreadyExist);
        }

        self.fee_tier_keys.push(fee_tier_key);
        Ok(())
    }

    pub fn remove(&mut self, fee_tier_key: FeeTierKey) -> Result<(), InvariantError> {
        let index = self
            .fee_tier_keys
            .iter()
            .position(|vec_fee_tier_key| *vec_fee_tier_key == fee_tier_key)
            .ok_or(InvariantError::FeeTierKeyNotFound)?;

        let length = self.fee_tier_keys.len();

        self.fee_tier_keys.swap(index, length - 1);
        self.fee_tier_keys.pop();

        Ok(())
    }

    pub fn contains(&self, fee_tier_key: FeeTierKey) -> bool {
        self.fee_tier_keys
            .iter()
            .any(|vec_fee_tier_key| *vec_fee_tier_key == fee_tier_key)
    }

    pub fn get_all(&self) -> Vec<FeeTierKey> {
        self.fee_tier_keys.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::percentage::Percentage;
    use decimal::*;

    #[ink::test]
    fn test_add() {
        let fee_tier_keys = &mut FeeTierKeys::default();
        let fee_tier_key = FeeTierKey::default();
        let new_fee_tier_key = FeeTierKey::new(Percentage::new(0), 2).unwrap();

        fee_tier_keys.add(fee_tier_key).unwrap();
        assert_eq!(fee_tier_keys.contains(fee_tier_key), true);
        assert_eq!(fee_tier_keys.contains(new_fee_tier_key), false);

        let result = fee_tier_keys.add(fee_tier_key);
        assert_eq!(result, Err(InvariantError::FeeTierKeyAlreadyExist));
    }

    #[ink::test]
    fn test_remove() {
        let fee_tier_keys = &mut FeeTierKeys::default();
        let fee_tier_key = FeeTierKey::default();

        fee_tier_keys.add(fee_tier_key).unwrap();

        fee_tier_keys.remove(fee_tier_key).unwrap();
        assert_eq!(fee_tier_keys.contains(fee_tier_key), false);

        let result = fee_tier_keys.remove(fee_tier_key);
        assert_eq!(result, Err(InvariantError::FeeTierKeyNotFound));
    }

    #[ink::test]
    fn test_get_all() {
        let fee_tier_keys = &mut FeeTierKeys::default();
        let fee_tier_key = FeeTierKey::default();
        let new_fee_tier_key = FeeTierKey::new(Percentage::new(0), 2).unwrap();

        let result = fee_tier_keys.get_all();
        assert_eq!(result, vec![]);
        assert_eq!(result.len(), 0);

        fee_tier_keys.add(fee_tier_key).unwrap();
        fee_tier_keys.add(new_fee_tier_key).unwrap();

        let result = fee_tier_keys.get_all();
        assert_eq!(result, vec![fee_tier_key, new_fee_tier_key]);
        assert_eq!(result.len(), 2);
    }
}
