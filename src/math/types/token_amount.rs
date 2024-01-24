use super::sqrt_price::SqrtPrice;
#[allow(unused_imports)]
use crate::alloc::string::ToString;
use core::convert::{TryFrom, TryInto};
use decimal::*;
#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};
use traceable_result::*;
#[cfg(feature = "wasm")]
use tsify::Tsify;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[decimal(0)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[cfg_attr(not(feature = "wasm"), derive(scale::Encode, scale::Decode))]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
#[cfg_attr(
    feature = "wasm",
    derive(Serialize, Deserialize, Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct TokenAmount(#[cfg_attr(feature = "wasm", tsify(type = "bigint"))] pub u128);

impl TokenAmount {
    pub fn from_big_sqrt_price(value: U256) -> TrackableResult<TokenAmount> {
        let result: u128 = value
            .checked_div(SqrtPrice::one())
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;

        Ok(TokenAmount(result))
    }

    pub fn from_big_sqrt_price_up(value: U256) -> TrackableResult<TokenAmount> {
        let result: u128 = value
            .checked_add(SqrtPrice::almost_one())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(SqrtPrice::one())
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;
        Ok(TokenAmount(result))
    }
}
