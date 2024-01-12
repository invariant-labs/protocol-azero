use crate::storage::pool_key::PoolKey;
use crate::storage::tick::Tick;
use crate::types::{
    fee_growth::{calculate_fee_growth_inside, FeeGrowth},
    fixed_point::FixedPoint,
    liquidity::Liquidity,
    percentage::Percentage,
    seconds_per_liquidity::SecondsPerLiquidity,
    sqrt_price::SqrtPrice,
    token_amount::TokenAmount,
};
use crate::{Pool, Position};
use decimal::*;
use traceable_result::{function, location, ok_or_mark_trace, trace};
// use paste::paste;
extern crate paste;
use js_sys::BigInt;
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

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmounts {
    pub x: TokenAmount,
    pub y: TokenAmount,
}

#[wasm_bindgen(js_name = "_simulateUnclaimedFees")]
pub fn simulate_unclaimed_fees(
    js_pool: JsValue,
    js_position: JsValue,
    js_lower_tick: JsValue,
    js_upper_tick: JsValue,
) -> Result<JsValue, JsValue> {
    let pool: Pool = convert!(js_pool)?;
    let position: Position = convert!(js_position)?;
    let lower_tick: Tick = convert!(js_lower_tick)?;
    let upper_tick: Tick = convert!(js_upper_tick)?;

    let (fee_growth_inside_x, fee_growth_inside_y) = calculate_fee_growth_inside(
        lower_tick.index,
        lower_tick.fee_growth_outside_x,
        lower_tick.fee_growth_outside_y,
        upper_tick.index,
        upper_tick.fee_growth_outside_x,
        upper_tick.fee_growth_outside_y,
        pool.current_tick_index,
        pool.fee_growth_global_x,
        pool.fee_growth_global_y,
    );

    let tokens_owed_x = ok_or_mark_trace!(fee_growth_inside_x
        .unchecked_sub(position.fee_growth_inside_x)
        .to_fee(position.liquidity))
    .map_err(|e| e.to_string())?;
    let tokens_owed_y = ok_or_mark_trace!(fee_growth_inside_y
        .unchecked_sub(position.fee_growth_inside_y)
        .to_fee(position.liquidity))
    .map_err(|e| e.to_string())?;

    Ok(serde_wasm_bindgen::to_value(&TokenAmounts {
        x: tokens_owed_x,
        y: tokens_owed_y,
    })?)
}
