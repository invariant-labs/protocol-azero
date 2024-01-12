use crate::math::{get_liquidity_by_x, get_liquidity_by_y};
use crate::types::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};

use crate::{convert, resolve};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "getLiquidityByX")]
pub fn wrapped_get_liquidity_by_x(
    js_x: JsValue,
    js_lower_tick: JsValue,
    js_upper_tick: JsValue,
    js_current_sqrt_price: JsValue,
    js_rounding_up: JsValue,
) -> Result<JsValue, JsValue> {
    let x: TokenAmount = convert!(js_x)?;
    let lower_tick: i64 = convert!(js_lower_tick)?;
    let upper_tick: i64 = convert!(js_upper_tick)?;
    let current_sqrt_price: SqrtPrice = convert!(js_current_sqrt_price)?;
    let rounding_up: bool = convert!(js_rounding_up)?;
    resolve!(get_liquidity_by_x(
        x,
        lower_tick as i32,
        upper_tick as i32,
        current_sqrt_price,
        rounding_up
    ))
}

#[wasm_bindgen(js_name = "getLiquidityByY")]
pub fn wrapped_get_liquidity_by_y(
    js_y: JsValue,
    js_lower_tick: JsValue,
    js_upper_tick: JsValue,
    js_current_sqrt_price: JsValue,
    js_rounding_up: JsValue,
) -> Result<JsValue, JsValue> {
    let y: TokenAmount = convert!(js_y)?;
    let lower_tick: i64 = convert!(js_lower_tick)?;
    let upper_tick: i64 = convert!(js_upper_tick)?;
    let current_sqrt_price: SqrtPrice = convert!(js_current_sqrt_price)?;
    let rounding_up: bool = convert!(js_rounding_up)?;
    resolve!(get_liquidity_by_y(
        y,
        lower_tick as i32,
        upper_tick as i32,
        current_sqrt_price,
        rounding_up
    ))
}
