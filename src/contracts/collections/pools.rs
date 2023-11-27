use crate::contracts::Pool;
use crate::contracts::PoolKey;
use crate::InvariantError;
use ink::storage::Mapping;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Pools {
    pools: Mapping<PoolKey, Pool>,
}

impl Pools {
    pub fn add(&mut self, pool_key: PoolKey, pool: &Pool) -> Result<(), InvariantError> {
        if self.pools.get(&pool_key).is_some() {
            return Err(InvariantError::PoolAlreadyExist);
        }

        self.pools.insert(pool_key, pool);
        Ok(())
    }

    pub fn update(&mut self, pool_key: PoolKey, pool: &Pool) -> Result<(), InvariantError> {
        self.pools
            .get(pool_key)
            .ok_or(InvariantError::PoolNotFound)?;

        self.pools.insert(pool_key, pool);
        Ok(())
    }

    pub fn remove(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
        self.pools
            .get(pool_key)
            .ok_or(InvariantError::PoolNotFound)?;

        self.pools.remove(&pool_key);
        Ok(())
    }

    pub fn get(&self, pool_key: PoolKey) -> Option<Pool> {
        let pool = self.pools.get(pool_key);
        pool
    }
}
