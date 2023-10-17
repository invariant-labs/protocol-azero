use crate::contracts::Tickmap;
use ink::{prelude::vec::Vec, primitives::AccountId};

#[derive(Debug, Default, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Tickmaps {
    // pub tickmaps: Vec<Tickmap>,
    pub test_tickmaps: Vec<Vec<AccountId>>,
}
