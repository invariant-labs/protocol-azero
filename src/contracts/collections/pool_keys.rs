use crate::contracts::{InvariantError, PoolKey, MAX_POOL_KEYS_RETURNED};
use alloc::vec::Vec;
use ink::storage::Mapping;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct PoolKeys {
    pool_keys: Mapping<PoolKey, u16>,
    pool_keys_by_index: Mapping<u16, PoolKey>,
    pool_keys_length: u16,
}

impl PoolKeys {
    pub fn add(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
        if self.contains(pool_key) {
            return Err(InvariantError::PoolKeyAlreadyExist);
        }

        self.pool_keys.insert(pool_key, &self.pool_keys_length);
        self.pool_keys_by_index
            .insert(self.pool_keys_length, &pool_key);
        self.pool_keys_length =
            self.pool_keys_length
                .checked_add(1)
                .ok_or(InvariantError::AddOverflow(
                    self.pool_keys_length as u128,
                    1,
                ))?;

        Ok(())
    }

    pub fn contains(&self, pool_key: PoolKey) -> bool {
        self.pool_keys.get(pool_key).is_some()
    }

    pub fn get_all(&self, size: u16, offset: u16) -> Vec<PoolKey> {
        let offset_with_size = offset.checked_add(size).unwrap();

        let upper_bound = if offset_with_size > self.pool_keys_length {
            self.pool_keys_length
        } else {
            offset_with_size
        };

        let max = if upper_bound.checked_sub(offset).unwrap() > MAX_POOL_KEYS_RETURNED {
            offset.checked_add(MAX_POOL_KEYS_RETURNED).unwrap()
        } else {
            upper_bound
        };

        (offset..max)
            .map(|index| self.pool_keys_by_index.get(index).unwrap())
            .collect()
    }

    pub fn count(&self) -> u16 {
        self.pool_keys_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{contracts::FeeTier, math::percentage::Percentage};
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

        let result = pool_keys.get_all(3, 0);
        assert_eq!(result, vec![]);
        assert_eq!(result.len(), 0);

        pool_keys.add(pool_key).unwrap();
        pool_keys.add(new_pool_key).unwrap();

        let result = pool_keys.get_all(3, 0);
        assert_eq!(result, vec![pool_key, new_pool_key]);
        assert_eq!(result.len(), 2);
    }
}
