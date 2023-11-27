use crate::contracts::Position;
use crate::InvariantError;
use ink::{prelude::vec::Vec, primitives::AccountId, storage::Mapping};

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Positions {
    positions_length: Mapping<AccountId, u32>,
    positions: Mapping<(AccountId, u32), Position>,
}

impl Positions {
    pub fn add(&mut self, account_id: AccountId, position: Position) -> Result<(), InvariantError> {
        let positions_length = self.get_length(account_id);
        let key = (account_id, positions_length);

        self.positions.insert(key, &position);

        self.positions_length
            .insert(account_id, &(positions_length + 1));

        Ok(())
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

        if index >= positions_length {
            return Err(InvariantError::PositionNotFound);
        }

        let position = self.positions.get((account_id, index));

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
        self.add(receiver_account_id, position)?;

        Ok(())
    }

    pub fn get_all(&self, account_id: AccountId) -> Vec<Position> {
        (0..self.get_length(account_id))
            .map(|index| self.positions.get((account_id, index)).unwrap())
            .collect()
    }

    pub fn get(&mut self, account_id: AccountId, index: u32) -> Result<Position, InvariantError> {
        let position = self
            .positions
            .get((account_id, index))
            .ok_or(InvariantError::PositionAlreadyExist)?;

        Ok(position)
    }

    pub fn get_length(&self, account_id: AccountId) -> u32 {
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
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        positions.add(account_id, position).unwrap();
        positions.add(account_id, new_position).unwrap();
        assert_eq!(positions.get(account_id, 0), Ok(position));
        assert_eq!(positions.get(account_id, 1), Ok(new_position));
        // assert_eq!(positions.get(account_id, 2), None);
        assert_eq!(positions.get_length(account_id), 2);
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

        positions.add(account_id, position).unwrap();

        positions.update(account_id, 0, &new_position).unwrap();
        assert_eq!(positions.get(account_id, 0), Ok(new_position));
        assert_eq!(positions.get_length(account_id), 1);

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

        positions.add(account_id, position).unwrap();
        positions.add(account_id, new_position).unwrap();

        let result = positions.remove(account_id, 0);
        assert_eq!(result, Ok(position));
        assert_eq!(positions.get(account_id, 0), Ok(new_position));
        assert_eq!(positions.get_length(account_id), 1);

        let result = positions.remove(account_id, 0);
        assert_eq!(result, Ok(new_position));
        // assert_eq!(positions.get(account_id, 0), None);
        assert_eq!(positions.get_length(account_id), 0);

        let result = positions.remove(account_id, 0);
        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }

    #[ink::test]
    fn test_transfer() {
        let positions = &mut Positions::default();
        let account_id = AccountId::from([0x01; 32]);
        let receiver_account_id = AccountId::from([0x02; 32]);
        let position = Position::default();

        positions.add(account_id, position).unwrap();

        positions
            .transfer(account_id, 0, receiver_account_id)
            .unwrap();
        // assert_eq!(positions.get(account_id, 0), None);
        assert_eq!(positions.get_length(account_id), 0);
        assert_eq!(positions.get(receiver_account_id, 0), Ok(position));
        assert_eq!(positions.get_length(receiver_account_id), 1);

        let result = positions.transfer(account_id, 0, receiver_account_id);
        assert_eq!(result, Err(InvariantError::PositionNotFound));
    }

    #[ink::test]
    fn test_get_all() {
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
        assert_eq!(positions.get_length(account_id), 0);

        positions.add(account_id, position).unwrap();
        positions.add(account_id, new_position).unwrap();

        let result = positions.get_all(account_id);
        assert_eq!(result, vec![position, new_position]);
        assert_eq!(result.len(), 2);
        assert_eq!(positions.get_length(account_id), 2);
    }

    #[ink::test]
    fn test_get_length() {
        let positions = &mut Positions::default();
        let account_id = AccountId::from([0x01; 32]);
        let position = Position::default();
        let new_position = Position {
            lower_tick_index: -1,
            upper_tick_index: 1,
            ..Position::default()
        };

        let result = positions.get_length(account_id);
        assert_eq!(result, 0);

        positions.add(account_id, position).unwrap();
        positions.add(account_id, new_position).unwrap();

        let result = positions.get_length(account_id);
        assert_eq!(result, 2);
    }
}
