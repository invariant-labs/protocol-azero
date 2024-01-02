use crate::alloc::string::ToString;
use crate::convert;
use crate::errors::InvariantError;
use crate::types::percentage::Percentage;
use decimal::*;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}

#[wasm_bindgen(js_name = "newFeeTier")]
pub fn new_fee_tier(js_fee: JsValue, js_tick_spacing: JsValue) -> Result<FeeTier, JsValue> {
    let fee: Percentage = convert!(js_fee)?;
    let tick_spacing: u16 = convert!(js_tick_spacing)?;

    if tick_spacing == 0 || tick_spacing > 100 {
        return Err(JsValue::from(
            InvariantError::InvalidTickSpacing.to_string(),
        ));
    }

    if fee > Percentage::from_integer(1) {
        return Err(JsValue::from(InvariantError::InvalidFee.to_string()));
    }

    Ok(FeeTier { fee, tick_spacing })
}
