use ink::primitives::AccountId;
use math::types::percentage::Percentage;

#[derive(PartialEq, Debug, scale::Decode, scale::Encode)]
pub struct State {
    pub admin: AccountId,
    pub protocol_fee: Percentage,
}

impl Default for State {
    fn default() -> Self {
        Self {
            admin: AccountId::from([0x0; 32]),
            protocol_fee: Default::default(),
        }
    }
}
