#[warn(unused_variables)]
#[warn(unused_imports)]
pub mod pair {
    use ink::{
        prelude::{vec, vec::Vec},
        primitives::{AccountId, Hash},
        storage::Mapping,
    };
    use openbrush::contracts::traits::psp22::PSP22Ref;
    use openbrush::{
        contracts::{ownable::*, psp22::*}, //
        modifiers,
        traits::{Balance, Storage, Timestamp},
    };

    use crate::{
        contract::OrderPair,
        contracts::logic::traits::pair::{Pair, PairError},
    };
    #[derive(scale::Decode, scale::Encode, Copy, Clone, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,)
    )]
    pub struct PairField {
        pub token_x: AccountId,
        pub token_y: AccountId,
        pub supply_x: Balance,
        pub supply_y: Balance,
    }

    impl Default for PairField {
        fn default() -> Self {
            PairField {
                token_x: AccountId::from([0x0; 32]),
                token_y: AccountId::from([0x0; 32]),
                supply_x: Default::default(),
                supply_y: Default::default(),
            }
        }
    }

    impl Pair for PairField {
        fn organize_tokens(&self, x: Balance, y: Balance) -> (Balance, Balance) {
            if x < y {
                (x, y)
            } else {
                (y, x)
            }
        }

        fn _mint(
            &mut self,
            user: AccountId,
            contract: AccountId,
            pair: OrderPair,
        ) -> Result<Balance, PairError> {
            self.supply_x += pair.x.1;
            self.supply_y += pair.y.1;
            // Transfer error should be handled
            PSP22Ref::transfer_from(&self.token_x, user, contract, pair.x.1, vec![]).unwrap();
            PSP22Ref::transfer_from(&self.token_y, user, contract, pair.y.1, vec![]).unwrap();
            Ok(pair.x.1 + pair.y.1)
        }

        fn _burn(
            &mut self,
            to: AccountId,
            contract: AccountId,
            amount: Balance,
        ) -> Result<(Balance, Balance), PairError> {
            if self.supply_x < amount || self.supply_y < amount {
                Err(PairError::InsufficientLiquidity)
            } else {
                self.supply_x -= amount;
                self.supply_y -= amount;
                Ok((amount, amount))
            }
        }

        fn _swap(
            &mut self,
            contract: AccountId,
            to: AccountId,
            amount: Balance,
            in_x: bool,
        ) -> Result<(Balance, Balance), PairError> {
            if in_x {
                if self.supply_y < amount {
                    Err(PairError::InsufficientLiquidity)
                } else {
                    let quote = self._quote(amount, self.supply_x, self.supply_y).unwrap();
                    self.supply_x += amount;
                    self.supply_y -= amount;

                    // transfer token_x from user to contract
                    PSP22Ref::transfer_from(&self.token_x, to, contract, amount, vec![]).unwrap();
                    // transfer token_y to user from contract
                    PSP22Ref::transfer(&self.token_y, to, amount, vec![]).unwrap();

                    Ok((self.supply_x, self.supply_y))
                }
            } else {
                if self.supply_x < amount {
                    Err(PairError::InsufficientLiquidity)
                } else {
                    let quote = self._quote(amount, self.supply_x, self.supply_y).unwrap();
                    self.supply_x -= amount;
                    self.supply_y += amount;

                    // transfer token_y from user
                    PSP22Ref::transfer_from(&self.token_y, to, contract, amount, vec![]).unwrap();
                    // transfer token_x to user
                    PSP22Ref::transfer(&self.token_x, to, amount, vec![]).unwrap();

                    Ok((self.supply_x, self.supply_y))
                }
            }
        }

        fn _quote(
            &mut self,
            amount: Balance,
            supply_0: Balance,
            supply_1: Balance,
        ) -> Result<u128, PairError> {
            Ok(1u128)
        }
    }
}
