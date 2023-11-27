use ink::storage::Mapping;

use crate::contracts::PoolKey;
use crate::contracts::Tick;
use crate::InvariantError;
#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Ticks {
    ticks: Mapping<(PoolKey, i32), Tick>,
}

impl Ticks {
    pub fn add(
        &mut self,
        pool_key: PoolKey,
        index: i32,
        tick: &Tick,
    ) -> Result<(), InvariantError> {
        self.ticks.insert(&(pool_key, index), tick);
        Ok(())
    }

    pub fn update(
        &mut self,
        pool_key: PoolKey,
        index: i32,
        tick: &Tick,
    ) -> Result<(), InvariantError> {
        self.ticks
            .get(&(pool_key, index))
            .ok_or(InvariantError::TickNotFound)?;

        self.ticks.insert((&pool_key, index), tick);
        Ok(())
    }

    pub fn remove(&mut self, pool_key: PoolKey, index: i32) -> Result<(), InvariantError> {
        self.ticks
            .get(&(pool_key, index))
            .ok_or(InvariantError::TickNotFound)?;

        self.ticks.remove(&(pool_key, index));
        Ok(())
    }

    pub fn get(&self, pool_key: PoolKey, index: i32) -> Option<Tick> {
        self.ticks.get(&(pool_key, index))
    }
}
