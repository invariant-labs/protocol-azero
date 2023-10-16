use ink::LangError;
use openbrush::{
    contracts::traits::{ownable::*, psp22::PSP22Error},
    traits::{AccountId, Balance, Timestamp},
};

use crate::contract::OrderPair;

#[openbrush::trait_definition]
pub trait Pair {
    fn organize_tokens(&self, x: Balance, y: Balance) -> (Balance, Balance);

    fn _mint(
        &mut self,
        user: AccountId,
        contract: AccountId,
        pair: OrderPair,
    ) -> Result<Balance, PairError>;

    fn _burn(
        &mut self,
        to: AccountId,
        contract: AccountId,
        amount: Balance,
    ) -> Result<(Balance, Balance), PairError>;

    fn _swap(
        &mut self,
        contract: AccountId,
        to: AccountId,
        amount: Balance,
        in_x: bool,
    ) -> Result<(Balance, Balance), PairError>;

    fn _quote(
        &mut self,
        amount: Balance,
        supply_0: Balance,
        supply_1: Balance,
    ) -> Result<u128, PairError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PairError {
    InsufficientLiquidity,
    TransferFailed,
}
