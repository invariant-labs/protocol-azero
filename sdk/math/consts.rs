use js_sys::BigInt;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;
pub const MAX_TICK: i32 = 221_818;
pub const MIN_TICK: i32 = -MAX_TICK;

pub const MAX_SQRT_PRICE: u128 = 65535383934512647000000000000;
pub const MIN_SQRT_PRICE: u128 = 15258932000000000000;

pub const TICK_SEARCH_RANGE: i32 = 256;

#[wasm_wrapper("getMaxTick")]
pub fn exported_get_max_tick() -> i32 {
    MAX_TICK
}

#[wasm_wrapper("getMinTick")]
pub fn exported_get_min_tick() -> i32 {
    MIN_TICK
}

#[wasm_wrapper]
pub fn get_max_sqrt_price() -> u128 {
    MAX_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_min_sqrt_price() -> u128 {
    MIN_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_tick_search_range() -> i32 {
    TICK_SEARCH_RANGE
}
