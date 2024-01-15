use crate::denominator;
use crate::scale;
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
#[decimal(12)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FixedPoint {
    #[tsify(type = "bigint")]
    pub v: u128,
}

scale!(FixedPoint);
denominator!(FixedPoint);
