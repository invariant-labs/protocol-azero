use crate::{contracts::PoolKey, InvariantError};
use alloc::vec::Vec;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct PoolKeys {
    pool_keys: Vec<PoolKey>,
}

impl PoolKeys {
    pub fn add(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
        if self.contains(pool_key) {
            return Err(InvariantError::PoolKeyAlreadyExist);
        }

        self.pool_keys.push(pool_key);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
        let index = self
            .pool_keys
            .iter()
            .position(|vec_pool_key| *vec_pool_key == pool_key)
            .ok_or(InvariantError::PoolKeyNotFound)?;

        self.pool_keys.remove(index);

        Ok(())
    }

    pub fn contains(&self, fee_tier_key: PoolKey) -> bool {
        self.pool_keys.contains(&fee_tier_key)
    }

    pub fn get_all(&self) -> Vec<PoolKey> {
        self.pool_keys.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::FeeTier;
    use math::percentage::Percentage;

    use decimal::*;
    use ink::primitives::AccountId;

    #[ink::test]
    fn test_add() {
        let pool_keys = &mut PoolKeys::default();
        let pool_key = PoolKey::default();
        let token_x = AccountId::from([1; 32]);
        let token_y = AccountId::from([2; 32]);
        let fee_tier = FeeTier {
            fee: Percentage::new(0),
            tick_spacing: 1,
        };
        let new_pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

        pool_keys.add(pool_key).unwrap();
        assert!(pool_keys.contains(pool_key));
        assert!(!pool_keys.contains(new_pool_key));

        let result = pool_keys.add(pool_key);
        assert_eq!(result, Err(InvariantError::PoolKeyAlreadyExist));
    }

    #[ink::test]
    fn test_remove() {
        let pool_keys = &mut PoolKeys::default();
        let pool_key = PoolKey::default();

        pool_keys.add(pool_key).unwrap();

        pool_keys.remove(pool_key).unwrap();
        assert!(!pool_keys.contains(pool_key));

        let result = pool_keys.remove(pool_key);
        assert_eq!(result, Err(InvariantError::PoolKeyNotFound));
    }

    #[ink::test]
    fn test_get_all() {
        let pool_keys = &mut PoolKeys::default();
        let pool_key = PoolKey::default();
        let token_x = AccountId::from([1; 32]);
        let token_y = AccountId::from([2; 32]);
        let fee_tier = FeeTier {
            fee: Percentage::new(0),
            tick_spacing: 1,
        };
        let new_pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

        let result = pool_keys.get_all();
        assert_eq!(result, vec![]);
        assert_eq!(result.len(), 0);

        pool_keys.add(pool_key).unwrap();
        pool_keys.add(new_pool_key).unwrap();

        let result = pool_keys.get_all();
        assert_eq!(result, vec![pool_key, new_pool_key]);
        assert_eq!(result.len(), 2);
    }
}
