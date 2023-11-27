use crate::{contracts::Position, InvariantError};
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
    ) -> Result<(), InvariantError> {
        let positions_length = self.get_length(account_id);

        if index >= positions_length {
            return Err(InvariantError::PositionNotFound);
        }

        self.positions.insert((account_id, index), position);

        Ok(())
    }

    pub fn remove(
        &mut self,
        account_id: AccountId,
        index: u32,
    ) -> Result<Position, InvariantError> {
        let positions_length = self.get_length(account_id);

        let position = self.positions.get((account_id, index));

        if position.is_none() {
            return Err(InvariantError::PositionNotFound);
        }

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

        Ok(position.unwrap())
    }

    pub fn transfer(
        &mut self,
        account_id: AccountId,
        index: u32,
        receiver_account_id: AccountId,
    ) -> Result<(), InvariantError> {
        let position = self.remove(account_id, index)?;
        self.add(receiver_account_id, position);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[ink::test]
    fn test_add() {
        let positions = &mut Positions::default();
        let account_id = AccountId::from([0x01; 32]);
        let position = Position::default();

        positions.add(account_id, position);

        let result = positions.get(account_id, 0);
        assert_eq!(result, Some(position));
        let result = positions.get(account_id, 1);
        assert_eq!(result, None);
    }

    #[ink::test]
    fn test_update() {
        let positions = &mut Positions::default();
        let account_id = AccountId::from([0x01; 32]);
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        positions.add(account_id, position);

        let result = positions.update(account_id, 0, &new_position);

        assert_eq!(result, Ok(()));
        let result = positions.get(account_id, 0);
        assert_eq!(result, Some(new_position));

        let result = positions.update(account_id, 1, &new_position);

        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }

    #[ink::test]
    fn test_remove() {
        let positions = &mut Positions::default();
        let account_id = AccountId::from([0x01; 32]);
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        positions.add(account_id, position);

        positions.add(account_id, new_position);

        let result = positions.remove(account_id, 0);

        assert_eq!(result, Ok(position));
        let result = positions.get(account_id, 0);
        assert_eq!(result, Some(new_position));

        let result = positions.remove(account_id, 0);

        assert_eq!(result, Ok(new_position));
        let result = positions.get(account_id, 0);
        assert_eq!(result, None);

        let result = positions.remove(account_id, 0);

        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }

    #[ink::test]
    fn test_transfer() {
        let positions = &mut Positions::default();
        let account_id = AccountId::from([0x01; 32]);
        let receiver_account_id = AccountId::from([0x02; 32]);
        let position = Position::default();

        positions.add(account_id, position);

        let result = positions.transfer(account_id, 0, receiver_account_id);

        assert_eq!(result, Ok(()));
        let result = positions.get(account_id, 0);
        assert_eq!(result, None);
        let result = positions.get(receiver_account_id, 0);
        assert_eq!(result, Some(position));

        let result = positions.transfer(account_id, 0, receiver_account_id);

        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }

    #[ink::test]
    fn get_all() {
        let positions = &mut Positions::default();
        let account_id = AccountId::from([0x01; 32]);
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        let result = positions.get_all(account_id);
        assert_eq!(result, vec![]);
        assert_eq!(result.len(), 0);

        positions.add(account_id, position);

        positions.add(account_id, new_position);

        let result = positions.get_all(account_id);
        assert_eq!(result, vec![position, new_position]);
        assert_eq!(result.len(), 2);
    }
}
