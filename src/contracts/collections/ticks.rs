use ink::storage::Mapping;

use crate::contracts::PoolKey;
use crate::contracts::Tick;
use crate::ContractErrors;
#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Ticks {
    ticks: Mapping<(PoolKey, i32), Tick>,
}

impl Ticks {
    pub fn get_tick(&self, key: PoolKey, index: i32) -> Option<Tick> {
        self.ticks.get(&(key, index))
    }
    // pub fn update_tick(&mut self, key: PoolKey, index: i32, tick: Tick) {}
    pub fn remove_tick(&mut self, key: PoolKey, index: i32) {
        self.ticks.remove(&(key, index));
    }

    pub fn add_tick(&mut self, key: PoolKey, index: i32, tick: Tick) {
        self.ticks.insert(&(key, index), &tick);
    }

    pub fn update_tick(
        &mut self,
        key: PoolKey,
        index: i32,
        tick: &Tick,
    ) -> Result<(), ContractErrors> {
        self.ticks
            .get(&(key, index))
            .ok_or(ContractErrors::TickNotFound)?;

        self.ticks.insert((&key, index), tick);

        Ok(())
    }
}
