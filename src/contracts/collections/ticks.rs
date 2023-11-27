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
    pub fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        let tick = self
            .ticks
            .get(&(key, index))
            .ok_or(InvariantError::TickNotFound)?;
        Ok(tick)
    }
    // pub fn update_tick(&mut self, key: PoolKey, index: i32, tick: Tick) {}
    pub fn remove_tick(&mut self, key: PoolKey, index: i32) -> Result<(), InvariantError> {
        self.ticks
            .get(&(key, index))
            .ok_or(InvariantError::TickNotFound)?;

        self.ticks.remove(&(key, index));
        Ok(())
    }

    pub fn add_tick(&mut self, key: PoolKey, index: i32, tick: Tick) -> Result<(), InvariantError> {
        if self.ticks.get(&(key, index)).is_some() {
            return Err(InvariantError::TickAlreadyExist);
        }

        self.ticks.insert(&(key, index), &tick);
        Ok(())
    }

    pub fn update_tick(
        &mut self,
        key: PoolKey,
        index: i32,
        tick: &Tick,
    ) -> Result<(), InvariantError> {
        // self.ticks
        //     .get(&(key, index))
        //     .ok_or(InvariantError::TickNotFound)?;

        self.ticks.insert((&key, index), tick);

        Ok(())
    }
}
