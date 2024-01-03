use crate::types::{
    fee_growth::FeeGrowth, fixed_point::FixedPoint, liquidity::Liquidity, percentage::Percentage,
    seconds_per_liquidity::SecondsPerLiquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};

use decimal::*;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AmountDeltaResult {
    pub x: TokenAmount,
    pub y: TokenAmount,
    pub update_liquidity: bool,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DecimalScales {
    sqrt_price: u8,
    token: u8,
    liquidity: u8,
    fixed_point: u8,
    fee_growth: u8,
    percentage: u8,
    seconds_per_liquidity: u8,
}

#[macro_export]
macro_rules! convert {
    ($value:expr) => {{
        serde_wasm_bindgen::from_value($value)
    }};
}

#[macro_export]
macro_rules! resolve {
    ($result:expr) => {{
        match $result {
            Ok(value) => Ok(value),
            Err(error) => Err(JsValue::from_str(&error.to_string())),
        }
    }};
}

#[wasm_bindgen(js_name = "getDecimalScales")]
pub fn get_decimal_scales() -> DecimalScales {
    DecimalScales {
        sqrt_price: SqrtPrice::scale(),
        token: TokenAmount::scale(),
        liquidity: Liquidity::scale(),
        fixed_point: FixedPoint::scale(),
        fee_growth: FeeGrowth::scale(),
        percentage: Percentage::scale(),
        seconds_per_liquidity: SecondsPerLiquidity::scale(),
    }
}
