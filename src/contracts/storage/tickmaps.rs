use crate::contracts::Tickmap;
use ink::prelude::vec::Vec;

// #[derive(Debug, scale::Decode, scale::Encode)]
// #[cfg_attr(
//     feature = "std",
//     derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
// )]
pub struct Tickmaps {
    pub tickmaps: Vec<Tickmap>,
}
