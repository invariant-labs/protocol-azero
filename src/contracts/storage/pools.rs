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
    pools: Mapping<PoolKey, (Pool, u64)>, //
}

impl Pools {
    pub fn get_pool(&self, key: PoolKey) -> Option<(Pool, u64)> {
        self.pools.get(&key)
    }
    pub fn add_pool(&mut self, key: PoolKey, pool: Pool, tickmap_index: u64) {
        self.pools.insert(&key, &(pool, tickmap_index));
    }
    pub fn remove_pool(&mut self, key: PoolKey) {}
}
