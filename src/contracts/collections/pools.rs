use crate::contracts::Pool;
use crate::contracts::PoolKey;
use crate::ContractErrors;
use ink::storage::Mapping;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Pools {
    pools: Mapping<PoolKey, Pool>,
}

impl Pools {
    pub fn create(&mut self, pool_key: PoolKey, pool: &Pool) -> Result<(), ContractErrors> {
        if self.pools.get(&pool_key).is_some() {
            return Err(ContractErrors::PoolAlreadyExist);
        }

        self.pools.insert(pool_key, pool);
        Ok(())
    }

    pub fn get(&self, pool_key: PoolKey) -> Result<Pool, ContractErrors> {
        let pool = self
            .pools
            .get(pool_key)
            .ok_or(ContractErrors::PoolNotFound)?;

        Ok(pool)
    }

    pub fn update(&mut self, pool_key: PoolKey, pool: &Pool) -> Result<(), ContractErrors> {
        self.pools
            .get(pool_key)
            .ok_or(ContractErrors::PoolNotFound)?;

        self.pools.insert(pool_key, pool);
        Ok(())
    }

    pub fn remove(&mut self, pool_key: PoolKey) -> Result<(), ContractErrors> {
        self.pools
            .get(pool_key)
            .ok_or(ContractErrors::PoolNotFound)?;

        self.pools.remove(&pool_key);
        Ok(())
    }
}
