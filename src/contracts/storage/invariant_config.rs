use crate::math::types::percentage::Percentage;
use ink::primitives::AccountId;
use uint::construct_uint;

#[ink::storage_item]
#[derive(Debug)]
pub struct InvariantConfig {
    pub admin: AccountId,
    pub protocol_fee: Percentage,
    pub poc_field: PocType,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            admin: AccountId::from([0x0; 32]),
            protocol_fee: Default::default(),
            poc_field: PocType(U256T::from(0)),
        }
    }
}

construct_uint! {
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct U256T(4);
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct PocType(pub U256T);
