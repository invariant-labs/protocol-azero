use crate::scale;
use core::convert::{TryFrom, TryInto};
use decimal::*;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(6)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Liquidity {
    #[tsify(type = "bigint")]
    pub v: u128,
}
scale!(Liquidity);
