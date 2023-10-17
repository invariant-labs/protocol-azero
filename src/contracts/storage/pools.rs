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

impl PoolKey {
    pub fn new(token_0: AccountId, token_1: AccountId, fee_tier: FeeTier) -> Self {
        if token_1 > token_0 {
            PoolKey(token_0, token_1, fee_tier)
        } else {
            PoolKey(token_1, token_0, fee_tier)
        }
    }
}

// #[ink::storage_item]
// #[derive(Debug, Default)]
// pub struct Pools {
//     pools: Mapping<PoolKey, PoolValue>, // (Pool, Tickmap)
// }
// impl Pools {
//     pub fn get_pool(&self, key: PoolKey) -> Option<PoolValue> {
//         self.pools.get(&key)
//     }
//     pub fn add_pool(&mut self, key: PoolKey, pool: Pool, tickmap: Tickmap) {
//         self.pools.insert(&key, &(1u128));
//     }
//     pub fn remove_pool(&mut self, key: PoolKey) {}
// }

// #[derive(Default, Debug, PartialEq, Copy, Clone, scale::Decode, scale::Encode)]
// #[cfg_attr(
//     feature = "std",
//     derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
// )]
// #[ink::storage_item]
// pub struct PoolValue(u128);

// impl scale::EncodeLike<PoolValue> for PoolValue {}
