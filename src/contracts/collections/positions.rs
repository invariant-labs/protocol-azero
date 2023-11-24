use crate::{contracts::Position, ContractErrors};
use ink::{
    prelude::{vec, vec::Vec},
    primitives::AccountId,
    storage::Mapping,
};

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Positions {
    positions_length: Mapping<AccountId, u32>,
    positions: Mapping<(AccountId, u32), Position>,
}

impl scale::EncodeLike<Vec<Position>> for Position {}

impl Positions {
    pub fn add(&mut self, account_id: AccountId, position: Position) {
        let positions_length = self.get_length(account_id);

        self.positions
            .insert((account_id, positions_length), &position);

        self.positions_length
            .insert(account_id, &(positions_length + 1));
    }

    pub fn update(
        &mut self,
        account_id: AccountId,
        index: u32,
        position: &Position,
    ) -> Result<(), ContractErrors> {
        let positions_length = self.get_length(account_id);

        if index >= positions_length {
            return Err(ContractErrors::PositionNotFound);
        }

        self.positions.insert((account_id, index), position);

        Ok(())
    }

    pub fn remove(
        &mut self,
        account_id: AccountId,
        index: u32,
    ) -> Result<Position, ContractErrors> {
        let positions_length = self.get_length(account_id);

        if index >= positions_length {
            return Err(ContractErrors::PositionNotFound);
        }

        let position = self.positions.get((account_id, index)).unwrap_or_default();

        if index < positions_length - 1 {
            let last_position = self
                .positions
                .take((account_id, positions_length - 1))
                .unwrap();
            self.positions.insert((account_id, index), &last_position);
        } else {
            self.positions.remove((account_id, index));
        }

        self.positions_length
            .insert(account_id, &(positions_length - 1));

        Ok(position)
    }

    pub fn transfer(
        &mut self,
        account_id: AccountId,
        index: u32,
        receiver: AccountId,
    ) -> Result<(), ContractErrors> {
        let position = self.remove(account_id, index)?;
        self.add(receiver, position);

        Ok(())
    }

    pub fn get_all(&self, account_id: AccountId) -> Vec<Position> {
        let mut positions = vec![];

        for index in 0..self.get_length(account_id) {
            let position = self.positions.get((account_id, index)).unwrap_or_default();
            positions.push(position);
        }

        positions
    }

    pub fn get(&mut self, account_id: AccountId, index: u32) -> Option<Position> {
        let position = self.positions.get((account_id, index));
        position
    }

    fn get_length(&self, account_id: AccountId) -> u32 {
        let positions_length = self.positions_length.get(account_id).unwrap_or(0);
        positions_length
    }
}
