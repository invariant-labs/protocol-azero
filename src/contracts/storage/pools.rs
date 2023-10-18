use ink::storage::Mapping;
// use ink::primitives::AccountId;
use openbrush::traits::AccountId;

use crate::contracts::FeeTier;
use crate::contracts::Pool;
// use crate::types::{pool::Pool, tickmap::Tickmap};
// use crate::contracts::Tickmap;

#[derive(scale::Decode, scale::Encode, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
// key (x: AccountId, y: AccountId, feeTier: FeeTier)
pub struct PoolKey(pub AccountId, pub AccountId, pub FeeTier);

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Pools {
    pools: Mapping<PoolKey, Pool>, //
}

impl Pools {
    pub fn get_pool(&self, key: PoolKey) -> Option<Pool> {
        self.pools.get(&key)
    }
    pub fn add_pool(&mut self, key: PoolKey, pool: Pool) {
        self.pools.insert(&key, &pool);
    }
    pub fn remove_pool(&mut self, key: PoolKey) {
        self.pools.remove(&key);
    }
}
