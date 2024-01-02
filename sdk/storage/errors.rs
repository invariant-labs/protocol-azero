use crate::alloc::string::ToString;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum InvariantError {
    InvalidTickSpacing,
    InvalidFee,
    TokensAreSame,
}

impl core::fmt::Display for InvariantError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "An invariant error occurred")
    }
}
