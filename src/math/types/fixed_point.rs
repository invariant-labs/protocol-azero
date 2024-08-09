use decimal::*;

// SqrtPrice with U256 underneath to avoid casting in calculate sqrt_price when calculating it from_tick
#[decimal(24)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct FixedPoint(pub U256);
