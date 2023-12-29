use super::sqrt_price::SqrtPrice;
use crate::alloc::string::ToString;
use core::convert::{TryFrom, TryInto};
use decimal::*;
use traceable_result::*;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(0)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TokenAmount(#[tsify(type = "BigInt")] pub u128);

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
