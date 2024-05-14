use crate::{Pool, Tick};
use tsify::Tsify;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
struct ActiveTick(#[tsify(type = "bigint")] u64, #[tsify(type = "bigint")] u64);

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
struct Tickmap(Vec<ActiveTick>);

#[wasm_bindgen(js_name = "simulateInvariantSwap", )]
pub fn simulate_invariant_swap(tickmap: Tickmap, ticks: JsValue, pool: JsValue)->String {
  //let tickmap = serde_wasm_bindgen::from_value::<Tickmap>(tickmap);
  
  format!("{:?}", tickmap)
}