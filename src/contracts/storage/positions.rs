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
    pub fn add_position(&mut self, caller: AccountId) {
        let positions_length = &self.get_length(caller);
        match positions_length {
            Some(_x) => {
                let mut current_positions = self.positions.get(caller).unwrap();
                let next_index = (current_positions.0 + 1) as u8;
                current_positions.0 += 1;
                current_positions.1.push(Position {
                    ..Default::default()
                });
                self.positions.insert(caller, &current_positions);
            }
            None => {
                let new_position = (
                    0,
                    vec![Position {
                        ..Default::default()
                    }],
                );
                self.positions.insert(caller, &new_position);
            }
        }
    }
    pub fn remove_position(&mut self, caller: AccountId, index: u32) {
        let positions_length = self.get_length(caller);

        if let Some(x) = positions_length {
            if x >= index {
                let mut current_positions = self.positions.get(caller).unwrap();
                current_positions.0 -= 1;
                current_positions.1.remove(index as usize);
                self.positions.insert(caller, &current_positions);
            }
        }
    }

    pub fn get_all_positions(&self, caller: AccountId) -> Vec<Position> {
        self.positions.get(&caller).unwrap_or_default().1
    }

    pub fn get_position(&mut self, caller: AccountId, index: u32) -> Option<Position> {
        let positions_length = self.get_length(caller);
        match positions_length {
            Some(x) => {
                if index <= x {
                    let positions = self.positions.get(caller).unwrap().1;
                    positions.get(index as usize).cloned()
                } else {
                    None
                }
            }
            None => None,
        }
    }
    fn get_length(&mut self, caller: AccountId) -> Option<u32> {
        let positions = self.positions.get(&caller);
        match positions {
            Some(x) => Some(x.0),
            None => None,
        }
    }
}
