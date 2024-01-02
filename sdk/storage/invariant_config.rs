use crate::alloc::string::String;
use crate::alloc::string::ToString;
use math::types::percentage::Percentage;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
#[derive(Debug, Clone, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct InvariantConfig {
    pub admin: String,
    pub protocol_fee: Percentage,
}
