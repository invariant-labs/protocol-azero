use ink::storage::Mapping;
use openbrush::traits::{AccountId, Balance};

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Balances {
    pub v: Mapping<(AccountId, AccountId, AccountId), Balance>,
}
