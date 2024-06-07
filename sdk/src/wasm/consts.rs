use crate::types::sqrt_price::get_max_tick;
use js_sys::BigInt;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;
pub const MAX_TICK: i32 = 221_818;
pub const MIN_TICK: i32 = -MAX_TICK;

pub const MAX_SQRT_PRICE: u128 = 65535383934512647000000000000;
pub const MIN_SQRT_PRICE: u128 = 15258932000000000000;

pub const TICK_SEARCH_RANGE: i32 = 256;
pub const CHUNK_SIZE: i32 = 64;

pub const MAX_TICK_CROSS: i32 = 173;

pub const MAX_RESULT_SIZE: usize = 16 * 1024 * 8;
pub const MAX_TICKMAP_QUERY_SIZE: usize = MAX_RESULT_SIZE / (16 + 64);

pub const LIQUIDITY_TICK_LIMIT: usize = MAX_RESULT_SIZE / (32 + 128 + 8);

pub const MAX_POOL_KEYS_RETURNED: u16 = 220;

pub const MAX_POOL_PAIRS_RETURNED: usize =
    MAX_RESULT_SIZE / (128 + 128 + 32 + 128 + 128 + 128 + 128 + 64 + 64 + 32 + 64 + 16);

#[wasm_wrapper]
pub fn get_global_max_sqrt_price() -> u128 {
    MAX_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_global_min_sqrt_price() -> u128 {
    MIN_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_tick_search_range() -> i32 {
    TICK_SEARCH_RANGE
}

#[wasm_wrapper]
pub fn get_max_chunk(tick_spacing: u16) -> u16 {
    let max_tick = get_max_tick(tick_spacing);
    let max_bitmap_index = (max_tick + MAX_TICK) / tick_spacing as i32;
    let max_chunk_index = max_bitmap_index / CHUNK_SIZE;
    max_chunk_index as u16
}

#[wasm_wrapper]
pub fn get_chunk_size() -> i32 {
    CHUNK_SIZE
}

#[wasm_wrapper]
pub fn get_max_tick_cross() -> i32 {
    MAX_TICK_CROSS
}

#[wasm_wrapper]
pub fn get_max_tickmap_query_size() -> u64 {
    MAX_TICKMAP_QUERY_SIZE as u64
}

#[wasm_wrapper]
pub fn get_liquidity_ticks_limit() -> u64 {
    LIQUIDITY_TICK_LIMIT as u64
}

#[wasm_wrapper]
pub fn get_max_pool_keys_returned() -> u16 {
    MAX_POOL_KEYS_RETURNED
}

#[wasm_wrapper]
pub fn get_max_pool_pairs_returned() -> u64 {
    MAX_POOL_PAIRS_RETURNED as u64
}
