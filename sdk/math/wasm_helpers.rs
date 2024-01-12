use crate::clamm::calculate_amount_delta;
use crate::storage::pool::Pool;
use crate::storage::pool_key::PoolKey;
use crate::storage::position::Position;
use crate::storage::tick::Tick;
use crate::types::{sqrt_price::SqrtPrice, token_amount::TokenAmount};

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
pub struct CalculateTokenAmounts {
    pub x: TokenAmount,
    pub y: TokenAmount,
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

#[wasm_bindgen(js_name = "calculateTokenAmountsFromPosition")]
pub fn calculate_token_amounts_from_position_liquidity(
    js_pool: JsValue,
    js_position: JsValue,
) -> Result<CalculateTokenAmounts, JsValue> {
    let pool: Pool = convert!(js_pool)?;
    let position: Position = convert!(js_position)?;

    let (x, y, _) = calculate_amount_delta(
        pool.current_tick_index as i32,
        pool.sqrt_price,
        position.liquidity,
        false,
        position.upper_tick_index as i32,
        position.lower_tick_index as i32,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let total_x = x + position.tokens_owed_x;
    let total_y = y + position.tokens_owed_y;
    Ok(CalculateTokenAmounts {
        x: total_x,
        y: total_y,
    })
}
