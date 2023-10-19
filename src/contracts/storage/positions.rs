use crate::contracts::Position;
use ink::{
    prelude::{vec, vec::Vec},
    storage::Mapping,
};
use openbrush::traits::AccountId;

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Positions {
    positions: Mapping<AccountId, (u32, Vec<Position>)>,
}

impl scale::EncodeLike<Vec<Position>> for Position {}

impl Positions {
    pub fn add(&mut self, account_id: AccountId, position: Position) {
        let (mut positions_length, mut positions) = self.get_value(account_id);

        if positions_length > 0 {
            positions_length += 1;
            positions.push(position);
            self.positions
                .insert(account_id, &(positions_length, positions));
        } else {
            self.positions.insert(account_id, &(1, vec![position]));
        }
    }

    pub fn remove(&mut self, account_id: AccountId, index: u32) {
        let (mut positions_length, mut positions) = self.get_value(account_id);

        if index < positions_length {
            positions_length -= 1;
            positions.remove(index as usize);
            self.positions
                .insert(account_id, &(positions_length, positions));
        }
    }

    pub fn get_all(&self, account_id: AccountId) -> Vec<Position> {
        self.positions.get(&account_id).unwrap_or_default().1
    }

    pub fn get(&mut self, account_id: AccountId, index: u32) -> Option<Position> {
        let positions_length = self.get_length(account_id);

        if index < positions_length {
            let positions = self.get_all(account_id);
            positions.get(index as usize).cloned()
        } else {
            None
        }
    }

    fn get_length(&self, account_id: AccountId) -> u32 {
        let positions = self.get_value(account_id);
        positions.0
    }

    fn get_value(&self, account_id: AccountId) -> (u32, Vec<Position>) {
        let positions = self.positions.get(&account_id).unwrap_or_default();
        positions
    }
}
