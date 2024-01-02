use crate::alloc::string::String;
use math::types::percentage::Percentage;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct InvariantConfig {
    admin: String,
    pub protocol_fee: Percentage,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            admin: String::from("0"),
            protocol_fee: Default::default(),
        }
    }
}
