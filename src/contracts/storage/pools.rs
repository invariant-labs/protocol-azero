use ink::storage::Mapping;
use openbrush::traits::AccountId;

use crate::contracts::FeeTier;
use crate::contracts::Pool;
use crate::contracts::PoolKey;
use crate::ContractErrors;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Pools {
    pools: Mapping<PoolKey, Pool>, //
}

impl Pools {
    pub fn add_pool(
        &mut self,
        key: PoolKey,
        timestamp: u64,
        admin: AccountId,
        init_tick: i32,
    ) -> Result<(), ContractErrors> {
        let pool = self.pools.get(&key);

        if pool.is_some() {
            return Err(ContractErrors::PoolAlreadyExist);
        }

        self.pools
            .insert(key, &Pool::create(init_tick, timestamp, admin));

        Ok(())
    }
    pub fn get_pool(&self, key: PoolKey) -> Result<Pool, ContractErrors> {
        let pool_option = self.pools.get(key);

        if pool_option.is_none() {
            return Err(ContractErrors::PoolNotFound);
        }

        Ok(pool_option.unwrap())
    }
    pub fn remove_pool(&mut self, key: PoolKey) {
        self.pools.remove(&key);
    }
}
