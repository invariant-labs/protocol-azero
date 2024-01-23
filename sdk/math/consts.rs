use traceable_result::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_wrapper::wasm_wrapper;

use crate::types::sqrt_price::{get_max_tick, get_min_tick};
pub const MAX_TICK: i32 = 221_818;
pub const MIN_TICK: i32 = -MAX_TICK;

pub const MAX_SQRT_PRICE: u128 = 65535383934512647000000000000;
pub const MIN_SQRT_PRICE: u128 = 15258932000000000000;

pub const TICK_SEARCH_RANGE: i32 = 256;
pub const CHUNK_SIZE: i32 = 64;

pub fn max_tick() -> TrackableResult<i32> {
    Ok(MAX_TICK)
}

pub fn min_tick() -> TrackableResult<i32> {
    Ok(MIN_TICK)
}

#[wasm_wrapper("getMaxTick")]
pub fn exported_get_max_tick(tick_spacing: u16) -> TrackableResult<i32> {
    Ok(get_max_tick(tick_spacing))
}

#[wasm_wrapper("getMinTick")]
pub fn exported_get_min_tick(tick_spacing: u16) -> TrackableResult<i32> {
    Ok(get_min_tick(tick_spacing))
}

#[wasm_wrapper]
pub fn get_max_sqrt_price() -> TrackableResult<u128> {
    Ok(MAX_SQRT_PRICE)
}

#[wasm_wrapper]
pub fn get_min_sqrt_price() -> TrackableResult<u128> {
    Ok(MIN_SQRT_PRICE)
}

#[wasm_wrapper]
pub fn get_tick_search_range() -> TrackableResult<i32> {
    Ok(TICK_SEARCH_RANGE)
}

#[wasm_wrapper]
pub fn get_max_chunk(tick_spacing: u16) -> TrackableResult<u16> {
    let max_tick = get_max_tick(tick_spacing);
    let max_bitmap_index = (max_tick + MAX_TICK) / tick_spacing as i32;
    let max_chunk_index = max_bitmap_index / CHUNK_SIZE;
    Ok(max_chunk_index as u16)
}

#[wasm_wrapper]
pub fn get_chunk_size() -> TrackableResult<i32> {
    Ok(CHUNK_SIZE)
}
