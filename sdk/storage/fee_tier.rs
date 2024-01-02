use crate::alloc::string::ToString;
use crate::errors::InvariantError;
use decimal::*;
use math::types::percentage::Percentage;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}

impl Default for FeeTier {
    fn default() -> Self {
        Self {
            fee: Percentage::new(0),
            tick_spacing: 1,
        }
    }
}

#[wasm_bindgen]
impl FeeTier {
    #[wasm_bindgen(constructor)]
    pub fn new(fee: Percentage, tick_spacing: u16) -> Result<FeeTier, JsValue> {
        if tick_spacing == 0 || tick_spacing > 100 {
            return Err(JsValue::from(
                InvariantError::InvalidTickSpacing.to_string(),
            ));
        }

        if fee > Percentage::from_integer(1) {
            return Err(JsValue::from(InvariantError::InvalidFee.to_string()));
        }

        Ok(Self { fee, tick_spacing })
    }
}
