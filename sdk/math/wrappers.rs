use crate::clamm::{
    calculate_amount_delta, calculate_max_liquidity_per_tick, calculate_min_amount_out, check_tick,
    check_ticks, compute_swap_step, get_delta_x, get_delta_y, get_next_sqrt_price_from_input,
    get_next_sqrt_price_from_output, get_next_sqrt_price_x_up, get_next_sqrt_price_y_down,
    is_enough_amount_to_change_price, SwapResult,
};
use crate::types::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};

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

#[wasm_bindgen(js_name = "getDeltaY")]
pub fn wrapped_get_delta_y(
    js_sqrt_price_a: JsValue,
    js_sqrt_price_b: JsValue,
    js_liquidity: JsValue,
    js_rounding_up: JsValue,
    // sqrt_price_a: SqrtPrice,
    // sqrt_price_b: SqrtPrice,
    // liquidity: Liquidity,
    // rounding_up: bool,
) -> Result<TokenAmount, JsValue> {
    let sqrt_price_a: SqrtPrice = serde_wasm_bindgen::from_value(js_sqrt_price_a)?;
    let sqrt_price_b: SqrtPrice = serde_wasm_bindgen::from_value(js_sqrt_price_b)?;
    let liquidity: Liquidity = serde_wasm_bindgen::from_value(js_liquidity)?;
    let rounding_up: bool = serde_wasm_bindgen::from_value(js_rounding_up)?;

    match get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, rounding_up) {
        Ok(amount) => Ok(amount),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "getDeltaX")]
pub fn wrapped_get_delta_x(
    sqrt_price_a: SqrtPrice,
    sqrt_price_b: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> Result<TokenAmount, JsValue> {
    match get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, rounding_up) {
        Ok(amount) => Ok(amount),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "getNextSqrtPriceFromInput")]
pub fn wrapped_get_next_sqrt_price_from_input(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
) -> Result<SqrtPrice, JsValue> {
    match get_next_sqrt_price_from_input(starting_sqrt_price, liquidity, amount, x_to_y) {
        Ok(sqrt_price) => Ok(sqrt_price),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "getNextSqrtPriceFromOutput")]
pub fn wrapped_get_next_sqrt_price_from_output(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
) -> Result<SqrtPrice, JsValue> {
    match get_next_sqrt_price_from_output(starting_sqrt_price, liquidity, amount, x_to_y) {
        Ok(sqrt_price) => Ok(sqrt_price),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "getNextSqrtPriceXUp")]
pub fn wrapped_get_next_sqrt_price_x_up(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    x: TokenAmount,
    add_x: bool,
) -> Result<SqrtPrice, JsValue> {
    match get_next_sqrt_price_x_up(starting_sqrt_price, liquidity, x, add_x) {
        Ok(sqrt_price) => Ok(sqrt_price),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "getNextSqrtPriceYDown")]
pub fn wrapped_get_next_sqrt_price_y_down(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    y: TokenAmount,
    add_y: bool,
) -> Result<SqrtPrice, JsValue> {
    match get_next_sqrt_price_y_down(starting_sqrt_price, liquidity, y, add_y) {
        Ok(sqrt_price) => Ok(sqrt_price),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "calculateAmountDelta")]
pub fn wrapped_calculate_amount_delta(
    current_tick_index: i32,
    current_sqrt_price: SqrtPrice,
    liquidity_delta: Liquidity,
    liquidity_sign: bool,
    upper_tick: i32,
    lower_tick: i32,
) -> Result<AmountDeltaResult, JsValue> {
    match calculate_amount_delta(
        current_tick_index,
        current_sqrt_price,
        liquidity_delta,
        liquidity_sign,
        upper_tick,
        lower_tick,
    ) {
        Ok(result) => Ok(AmountDeltaResult {
            x: result.0,
            y: result.1,
            update_liquidity: result.2,
        }),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "isEnoughAmountToChangePrice")]
pub fn wrapped_is_enough_to_change_price(
    amount: TokenAmount,
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    fee: Percentage,
    by_amount_in: bool,
    x_to_y: bool,
) -> Result<bool, JsValue> {
    match is_enough_amount_to_change_price(
        amount,
        starting_sqrt_price,
        liquidity,
        fee,
        by_amount_in,
        x_to_y,
    ) {
        Ok(is_enough) => Ok(is_enough),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "calculateMaxLiquidityPerTick")]
pub fn wrapped_calculate_max_liquidity_per_tick(tick_spacing: u16) -> Result<Liquidity, JsValue> {
    Ok(calculate_max_liquidity_per_tick(tick_spacing))
}

#[wasm_bindgen(js_name = "checkTicks")]
pub fn wrapped_check_ticks(
    tick_lower: i32,
    tick_upper: i32,
    tick_spacing: u16,
) -> Result<(), JsValue> {
    match check_ticks(tick_lower, tick_upper, tick_spacing) {
        Ok(_) => Ok(()),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "checkTick")]
pub fn wrapped_check_tick(tick_index: i32, tick_spacing: u16) -> Result<(), JsValue> {
    match check_tick(tick_index, tick_spacing) {
        Ok(_) => Ok(()),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}

#[wasm_bindgen(js_name = "calculateMinAmountOut")]
pub fn wrapped_calculate_min_amount_out(
    expected_amount_out: TokenAmount,
    slippage: Percentage,
) -> Result<TokenAmount, JsValue> {
    Ok(calculate_min_amount_out(expected_amount_out, slippage))
}

#[wasm_bindgen(js_name = "computeSwapStep")]
pub fn wrapped_compute_swap_step(
    current_sqrt_price: SqrtPrice,
    target_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    by_amount_in: bool,
    fee: Percentage,
) -> Result<SwapResult, JsValue> {
    match compute_swap_step(
        current_sqrt_price,
        target_sqrt_price,
        liquidity,
        amount,
        by_amount_in,
        fee,
    ) {
        Ok(swap_result) => Ok(swap_result),
        Err(error) => Err(JsValue::from_str(&error.cause)),
    }
}
