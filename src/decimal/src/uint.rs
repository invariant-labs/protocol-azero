//! Large uint types

// required for clippy
#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::reversed_empty_ranges)]

use crate::{UintCast, UintCheckedCast};
use alloc::string::{String, ToString};
use decimal_core::impl_units_casts;

use num_traits::{WrappingAdd, WrappingSub};
use uint::construct_uint;

construct_uint! {
    pub struct U512(8);
}

construct_uint! {
    pub struct U448(7);
}

construct_uint! {
    pub struct U384(6);
}

construct_uint! {
    pub struct U320(5);
}

#[cfg(not(feature = "invariant-wasm"))]
construct_uint! {
    #[ink::scale_derive(Decode, Encode)]
    #[cfg_attr(feature = "std", derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct U256(4);
}

construct_uint! {
    pub struct U192(3);
}
construct_uint! {
    pub struct U128(2);
}

impl_units_casts!(U512 8, U448 7, U384 6, U320 5, U256 4, U192 3, U128 2);

macro_rules! impl_uint_casts_as_default_from {
    ($thing:ident, $from:ty) => {
        impl UintCast<$from> for $thing {
            fn uint_cast(value: $from) -> $thing {
                From::from(value)
            }
        }
        impl UintCheckedCast<$from> for $thing {
            fn uint_checked_cast(value: $from) -> Result<$thing, String> {
                Ok(From::from(value))
            }
        }
    };
}
impl_uint_casts_as_default_from!(u128, u8);
impl_uint_casts_as_default_from!(u128, u16);
impl_uint_casts_as_default_from!(u128, u32);
impl_uint_casts_as_default_from!(u128, u64);

impl_uint_casts_as_default_from!(u64, u8);
impl_uint_casts_as_default_from!(u64, u16);
impl_uint_casts_as_default_from!(u64, u32);

// i32 casts are added to allow not specifying the type in from_integer function in decimal factories
impl UintCast<i32> for u128 {
    fn uint_cast(value: i32) -> u128 {
        if value < 0 {
            panic!("Failed to cast i32 to u128")
        }
        From::from(value as u32)
    }
}

impl UintCast<i32> for u64 {
    fn uint_cast(value: i32) -> u64 {
        if value < 0 {
            panic!("Failed to cast i32 to u64")
        }
        From::from(value as u32)
    }
}

impl WrappingAdd for U256 {
    fn wrapping_add(&self, other: &Self) -> Self {
        if *other > U256::MAX - *self {
            (other - (U256::MAX - self)) - 1
        } else {
            self + other
        }
    }
}

impl WrappingSub for U256 {
    fn wrapping_sub(&self, other: &Self) -> Self {
        if other > self {
            U256::MAX - (other - self) + 1
        } else {
            self - other
        }
    }
}

#[cfg(feature = "invariant-wasm")]
pub mod invariant_wasm {
    use alloc::string::{String, ToString};
    use serde::{Deserialize, Serialize};
    use uint::construct_uint;
    construct_uint! {
        pub struct U256(4);
    }
    impl Serialize for U256 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for U256 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Self::from_dec_str(&s).expect("Failed to deserialize U256 from string"))
        }
    }
}
#[cfg(feature = "invariant-wasm")]
pub use invariant_wasm::*;

#[allow(dead_code)]
pub fn checked_u320_to_u256(n: U320) -> Option<U256> {
    if !(n >> 256).is_zero() {
        return None;
    }

    Some(U256([
        n.low_u64(),
        (n >> 64).low_u64(),
        (n >> 128).low_u64(),
        (n >> 192).low_u64(),
    ]))
}

#[allow(dead_code)]
pub fn u320_to_u256(n: U320) -> U256 {
    checked_u320_to_u256(n).unwrap()
}

#[allow(dead_code)]
pub const fn to_u256(n: u128) -> U256 {
    U256([n as u64, (n >> 64) as u64, 0, 0])
}

#[allow(dead_code)]
pub fn u256_to_u320(n: U256) -> U320 {
    U320([
        n.low_u64(),
        (n >> 64).low_u64(),
        (n >> 128).low_u64(),
        (n >> 192).low_u64(),
        0,
    ])
}

#[allow(dead_code)]
pub fn to_u320(n: u128) -> U320 {
    u256_to_u320(to_u256(n))
}
