use crate::alloc::string::ToString;
use crate::errors::InvariantError;
use crate::FeeTier;
use alloc::string::String;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)] //
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PoolKey {
    token_x: String,
    token_y: String,
    pub fee_tier: FeeTier,
}

impl PoolKey {
    // #[wasm_bindgen(constructor)]
    pub fn new(token_0: String, token_1: String, fee_tier: FeeTier) -> Result<PoolKey, JsValue> {
        if token_0 == token_1 {
            return Err(JsValue::from(InvariantError::TokensAreSame.to_string()));
        }

        if token_0 < token_1 {
            Ok(PoolKey {
                token_x: token_0,
                token_y: token_1,
                fee_tier,
            })
        } else {
            Ok(PoolKey {
                token_x: token_1,
                token_y: token_0,
                fee_tier,
            })
        }
    }
}
