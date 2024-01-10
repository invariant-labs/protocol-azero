use crate::clamm::{
    calculate_amount_delta, calculate_max_liquidity_per_tick, calculate_min_amount_out, check_tick,
    check_ticks, compute_swap_step, get_delta_x, get_delta_y, get_next_sqrt_price_from_input,
    get_next_sqrt_price_from_output, get_next_sqrt_price_x_up, get_next_sqrt_price_y_down,
    is_enough_amount_to_change_price, SwapResult,
};
use crate::math::{get_liquidity_by_x, get_liquidity_by_y, SingleTokenLiquidity};
use crate::types::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};

use crate::{convert, resolve, wasm_helpers::AmountDeltaResult};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "getDeltaY")]
pub fn wrapped_get_delta_y(
    js_sqrt_price_a: JsValue,
    js_sqrt_price_b: JsValue,
    js_liquidity: JsValue,
    js_rounding_up: JsValue,
) -> Result<TokenAmount, JsValue> {
    let sqrt_price_a: SqrtPrice = convert!(js_sqrt_price_a)?;
    let sqrt_price_b: SqrtPrice = convert!(js_sqrt_price_b)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let rounding_up: bool = convert!(js_rounding_up)?;
    resolve!(get_delta_y(
        sqrt_price_a,
        sqrt_price_b,
        liquidity,
        rounding_up
    ))
}

#[wasm_bindgen(js_name = "getDeltaX")]
pub fn wrapped_get_delta_x(
    js_sqrt_price_a: JsValue,
    js_sqrt_price_b: JsValue,
    js_liquidity: JsValue,
    js_rounding_up: JsValue,
) -> Result<TokenAmount, JsValue> {
    let sqrt_price_a: SqrtPrice = convert!(js_sqrt_price_a)?;
    let sqrt_price_b: SqrtPrice = convert!(js_sqrt_price_b)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let rounding_up: bool = convert!(js_rounding_up)?;
    resolve!(get_delta_x(
        sqrt_price_a,
        sqrt_price_b,
        liquidity,
        rounding_up,
    ))
}

#[wasm_bindgen(js_name = "getNextSqrtPriceFromInput")]
pub fn wrapped_get_next_sqrt_price_from_input(
    js_starting_sqrt_price: JsValue,
    js_liquidity: JsValue,
    js_amount: JsValue,
    js_x_to_y: JsValue,
) -> Result<SqrtPrice, JsValue> {
    let starting_sqrt_price: SqrtPrice = convert!(js_starting_sqrt_price)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let amount: TokenAmount = convert!(js_amount)?;
    let x_to_y: bool = convert!(js_x_to_y)?;
    resolve!(get_next_sqrt_price_from_input(
        starting_sqrt_price,
        liquidity,
        amount,
        x_to_y
    ))
}

#[wasm_bindgen(js_name = "getNextSqrtPriceFromOutput")]
pub fn wrapped_get_next_sqrt_price_from_output(
    js_starting_sqrt_price: JsValue,
    js_liquidity: JsValue,
    js_amount: JsValue,
    js_x_to_y: JsValue,
) -> Result<SqrtPrice, JsValue> {
    let starting_sqrt_price: SqrtPrice = convert!(js_starting_sqrt_price)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let amount: TokenAmount = convert!(js_amount)?;
    let x_to_y: bool = convert!(js_x_to_y)?;
    resolve!(get_next_sqrt_price_from_output(
        starting_sqrt_price,
        liquidity,
        amount,
        x_to_y
    ))
}

#[wasm_bindgen(js_name = "getNextSqrtPriceXUp")]
pub fn wrapped_get_next_sqrt_price_x_up(
    js_starting_sqrt_price: JsValue,
    js_liquidity: JsValue,
    js_x: JsValue,
    js_add_x: JsValue,
) -> Result<SqrtPrice, JsValue> {
    let starting_sqrt_price: SqrtPrice = convert!(js_starting_sqrt_price)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let x: TokenAmount = convert!(js_x)?;
    let add_x: bool = convert!(js_add_x)?;
    resolve!(get_next_sqrt_price_x_up(
        starting_sqrt_price,
        liquidity,
        x,
        add_x
    ))
}

#[wasm_bindgen(js_name = "getNextSqrtPriceYDown")]
pub fn wrapped_get_next_sqrt_price_y_down(
    js_starting_sqrt_price: JsValue,
    js_liquidity: JsValue,
    js_y: JsValue,
    js_add_y: JsValue,
) -> Result<SqrtPrice, JsValue> {
    let starting_sqrt_price: SqrtPrice = convert!(js_starting_sqrt_price)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let y: TokenAmount = convert!(js_y)?;
    let add_y: bool = convert!(js_add_y)?;
    resolve!(get_next_sqrt_price_y_down(
        starting_sqrt_price,
        liquidity,
        y,
        add_y
    ))
}

#[wasm_bindgen(js_name = "calculateAmountDelta")]
pub fn wrapped_calculate_amount_delta(
    js_current_tick_index: JsValue,
    js_current_sqrt_price: JsValue,
    js_liquidity_delta: JsValue,
    js_liquidity_sign: JsValue,
    js_upper_tick: JsValue,
    js_lower_tick: JsValue,
) -> Result<AmountDeltaResult, JsValue> {
    let current_tick_index: i64 = convert!(js_current_tick_index)?;
    let current_sqrt_price: SqrtPrice = convert!(js_current_sqrt_price)?;
    let liquidity_delta: Liquidity = convert!(js_liquidity_delta)?;
    let liquidity_sign: bool = convert!(js_liquidity_sign)?;
    let upper_tick: i64 = convert!(js_upper_tick)?;
    let lower_tick: i64 = convert!(js_lower_tick)?;
    match calculate_amount_delta(
        current_tick_index as i32,
        current_sqrt_price,
        liquidity_delta,
        liquidity_sign,
        upper_tick as i32,
        lower_tick as i32,
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
    js_amount: JsValue,
    js_starting_sqrt_price: JsValue,
    js_liquidity: JsValue,
    js_fee: JsValue,
    js_by_amount_in: JsValue,
    js_x_to_y: JsValue,
) -> Result<bool, JsValue> {
    let amount: TokenAmount = convert!(js_amount)?;
    let starting_sqrt_price: SqrtPrice = convert!(js_starting_sqrt_price)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let fee: Percentage = convert!(js_fee)?;
    let by_amount_in: bool = convert!(js_by_amount_in)?;
    let x_to_y: bool = convert!(js_x_to_y)?;
    resolve!(is_enough_amount_to_change_price(
        amount,
        starting_sqrt_price,
        liquidity,
        fee,
        by_amount_in,
        x_to_y
    ))
}

#[wasm_bindgen(js_name = "calculateMaxLiquidityPerTick")]
pub fn wrapped_calculate_max_liquidity_per_tick(
    js_tick_spacing: JsValue,
) -> Result<Liquidity, JsValue> {
    let tick_spacing: u16 = convert!(js_tick_spacing)?;
    Ok(calculate_max_liquidity_per_tick(tick_spacing))
}

#[wasm_bindgen(js_name = "checkTicks")]
pub fn wrapped_check_ticks(
    js_tick_lower: JsValue,
    js_tick_upper: JsValue,
    js_tick_spacing: JsValue,
) -> Result<(), JsValue> {
    let tick_lower: i64 = convert!(js_tick_lower)?;
    let tick_upper: i64 = convert!(js_tick_upper)?;
    let tick_spacing: u64 = convert!(js_tick_spacing)?;
    resolve!(check_ticks(
        tick_lower as i32,
        tick_upper as i32,
        tick_spacing as u16
    ))
}

#[wasm_bindgen(js_name = "checkTick")]
pub fn wrapped_check_tick(js_tick_index: JsValue, js_tick_spacing: JsValue) -> Result<(), JsValue> {
    let tick_index: i64 = convert!(js_tick_index)?;
    let tick_spacing: u64 = convert!(js_tick_spacing)?;
    resolve!(check_tick(tick_index as i32, tick_spacing as u16))
}

#[wasm_bindgen(js_name = "calculateMinAmountOut")]
pub fn wrapped_calculate_min_amount_out(
    js_expected_amount_out: JsValue,
    js_slippage: JsValue,
) -> Result<TokenAmount, JsValue> {
    let expected_amount_out: TokenAmount = convert!(js_expected_amount_out)?;
    let slippage: Percentage = convert!(js_slippage)?;
    Ok(calculate_min_amount_out(expected_amount_out, slippage))
}

#[wasm_bindgen(js_name = "computeSwapStep")]
pub fn wrapped_compute_swap_step(
    js_current_sqrt_price: JsValue,
    js_target_sqrt_price: JsValue,
    js_liquidity: JsValue,
    js_amount: JsValue,
    js_by_amount_in: JsValue,
    js_fee: JsValue,
) -> Result<SwapResult, JsValue> {
    let current_sqrt_price: SqrtPrice = convert!(js_current_sqrt_price)?;
    let target_sqrt_price: SqrtPrice = convert!(js_target_sqrt_price)?;
    let liquidity: Liquidity = convert!(js_liquidity)?;
    let amount: TokenAmount = convert!(js_amount)?;
    let by_amount_in: bool = convert!(js_by_amount_in)?;
    let fee: Percentage = convert!(js_fee)?;
    resolve!(compute_swap_step(
        current_sqrt_price,
        target_sqrt_price,
        liquidity,
        amount,
        by_amount_in,
        fee
    ))
}

#[wasm_bindgen(js_name = "_getLiquidityByX")]
pub fn wrapped_get_liquidity_by_x(
    js_x: JsValue,
    js_lower_tick: JsValue,
    js_upper_tick: JsValue,
    js_current_sqrt_price: JsValue,
    js_rounding_up: JsValue,
) -> Result<SingleTokenLiquidity, JsValue> {
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

#[wasm_bindgen(js_name = "_getLiquidityByY")]
pub fn wrapped_get_liquidity_by_y(
    js_y: JsValue,
    js_lower_tick: JsValue,
    js_upper_tick: JsValue,
    js_current_sqrt_price: JsValue,
    js_rounding_up: JsValue,
) -> Result<SingleTokenLiquidity, JsValue> {
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
