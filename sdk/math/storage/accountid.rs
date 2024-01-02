use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default, Serialize, Deserialize,
)]
pub struct AccountId(pub [u8; 32]);
