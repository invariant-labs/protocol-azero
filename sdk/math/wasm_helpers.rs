use crate::clamm::calculate_amount_delta;
use crate::storage::pool_key::PoolKey;
use crate::storage::tick::Tick;
use crate::types::{liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount};

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

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmounts {
    pub x: TokenAmount,
    pub y: TokenAmount,
}

// Logging to typescript
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_many(a: &str, b: &str);
}

#[macro_export]
macro_rules! scale {
    ($decimal:ident) => {
        ::paste::paste! {
            #[wasm_bindgen]
            #[allow(non_snake_case)]
            pub fn [<get $decimal Scale >] () -> BigInt {
                BigInt::from($decimal::scale())
            }
        }
    };
}

#[macro_export]
macro_rules! denominator {
    ($decimal:ident) => {
        ::paste::paste! {
            #[wasm_bindgen]
            #[allow(non_snake_case)]
            pub fn [<get $decimal Denominator >] () -> BigInt {
                BigInt::from(10u128.pow($decimal::scale() as u32))
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
            Ok(value) => Ok(serde_wasm_bindgen::to_value(&value)?),
            Err(error) => Err(JsValue::from_str(&error.to_string())),
        }
    }};
}

#[wasm_bindgen(js_name = "wrappedCalculateTokenAmounts")]
pub fn calculate_token_amounts(
    js_current_tick_index: JsValue,
    js_current_sqrt_price: JsValue,
    js_liquidity: JsValue,
    js_upper_tick_index: JsValue,
    js_lower_tick_index: JsValue,
) -> Result<JsValue, JsValue> {
    let current_tick_index: i64 = convert!(js_current_tick_index)?;
    let current_sqrt_price: SqrtPrice = convert!(js_current_sqrt_price)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let upper_tick_index: i64 = convert!(js_upper_tick_index)?;
    let lower_tick_index: i64 = convert!(js_lower_tick_index)?;

    let (x, y, _) = calculate_amount_delta(
        current_tick_index as i32,
        current_sqrt_price,
        liquidity,
        false,
        upper_tick_index as i32,
        lower_tick_index as i32,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(serde_wasm_bindgen::to_value(&TokenAmounts { x, y })?)
}
