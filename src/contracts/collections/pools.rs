use ink::storage::Mapping;
use openbrush::traits::AccountId;

use crate::contract::Contract;
use crate::contracts::FeeTier;
use crate::contracts::Pool;
use crate::contracts::PoolKey;
use crate::math::check_tick;
use crate::math::MAX_TICK;
use crate::ContractErrors;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Pools {
    pools: Mapping<PoolKey, Pool>, //
}

impl Pools {
    pub fn add_pool(
        &mut self,
        pool_key: PoolKey,
        current_timestamp: u64,
        fee_receiver: AccountId,
        init_tick: i32,
    ) -> Result<(), ContractErrors> {
        let pool = self.pools.get(&pool_key);

        check_tick(init_tick, pool_key.fee_tier.tick_spacing)
            .map_err(|_| ContractErrors::InvalidTickIndexOrTickSpacing)?;

        if pool.is_some() {
            return Err(ContractErrors::PoolAlreadyExist);
        }

        self.pools.insert(
            pool_key,
            &Pool::create(init_tick, current_timestamp, fee_receiver),
        );

        Ok(())
    }

    pub fn get_pool(&self, pool_key: PoolKey) -> Result<Pool, ContractErrors> {
        let pool_option = self.pools.get(pool_key);
        let pool = pool_option.ok_or(ContractErrors::PoolNotFound)?;
        Ok(pool)
    }

    pub fn remove_pool(&mut self, pool_key: PoolKey) {
        self.pools.remove(&pool_key);
    }

    pub fn change_fee_receiver(
        &mut self,
        pool_key: PoolKey,
        fee_receiver: AccountId,
    ) -> Result<(), ContractErrors> {
        let mut pool = self.get_pool(pool_key)?;
        pool.fee_receiver = fee_receiver;
        self.update_pool(pool_key, &pool);

        Ok(())
    }

    pub fn update_pool(&mut self, pool_key: PoolKey, pool: &Pool) {
        self.pools.insert(pool_key, pool);
    }
}
