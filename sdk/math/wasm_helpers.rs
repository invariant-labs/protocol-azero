use crate::storage::pool_key::PoolKey;
use crate::storage::tick::Tick;
use crate::types::{
    fee_growth::FeeGrowth, fixed_point::FixedPoint, liquidity::Liquidity, percentage::Percentage,
    seconds_per_liquidity::SecondsPerLiquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use decimal::*;
// use paste::paste;
extern crate paste;
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

#[derive(PartialEq, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SwapHop {
    pub pool_key: PoolKey,
    pub x_to_y: bool,
}

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub target_sqrt_price: SqrtPrice,
    pub ticks: Vec<Tick>,
}

#[macro_export]
macro_rules! scale {
    ($decimal:ident) => {
        ::paste::paste! {
            #[wasm_bindgen]
            #[allow(non_snake_case)]
            pub fn [<get $decimal Scale >] () -> u8 {
                $decimal::scale()
            }
        }
    };
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
