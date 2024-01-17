use wasm_bindgen::prelude::*;

pub const MAX_TICK: i32 = 221_818;
pub const MIN_TICK: i32 = -MAX_TICK;

pub const MAX_SQRT_PRICE: u128 = 65535383934512647000000000000;
pub const MIN_SQRT_PRICE: u128 = 15258932000000000000;

pub const TICK_SEARCH_RANGE: i32 = 256;

#[wasm_bindgen(typescript_custom_section)]
const ts_exports: &'static str = r#"
const maxTick = 221818n;
const minTick = -maxTick;
const maxSqrtPrice = 65535383934512647000000000000n;
const minSqrtPrice = 15258932000000000000n;
"#;
