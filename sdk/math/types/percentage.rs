use core::convert::{TryFrom, TryInto};
use decimal::*;

use crate::scale;
use serde::{Deserialize, Serialize};

use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(12)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Percentage {
    #[tsify(type = "BigInt")]
    pub v: u64,
}
scale!(Percentage);
