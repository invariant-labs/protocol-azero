use ink::primitives::AccountId;
use math::types::percentage::Percentage;

#[ink::storage_item]
#[derive(Debug)]
pub struct InvariantConfig {
    pub admin: AccountId,
    pub protocol_fee: Percentage,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            admin: AccountId::from([0x0; 32]),
            protocol_fee: Default::default(),
        }
    }
}
