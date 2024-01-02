use crate::alloc::string::ToString;
use crate::types::percentage::Percentage;
use crate::AccountId;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
#[derive(Debug, Clone, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct InvariantConfig {
    #[tsify(type = "number[]")]
    pub admin: AccountId,
    pub protocol_fee: Percentage,
}
