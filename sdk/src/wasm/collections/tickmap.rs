use serde::{Deserialize, Serialize};

use crate::consts::{CHUNK_SIZE, MAX_TICK, TICK_SEARCH_RANGE};
use crate::types::sqrt_price::{calculate_sqrt_price, SqrtPrice};
use crate::InvariantError;
use js_sys::*;
use std::collections::HashMap;
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

#[derive(Serialize, Deserialize, Clone, Debug, Tsify)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct Tickmap {
    #[tsify(type = "Map<bigint,bigint>")]
    pub bitmap: HashMap<u64, u64>,
}

impl Default for Tickmap {
    fn default() -> Self {
        let bitmap = HashMap::default();
        Tickmap { bitmap }
    }
}

#[wasm_wrapper("tickIndexToPosition")]
pub fn tick_to_position_js(tick: i32, tick_spacing: u16) -> TrackableResult<(u16, u8)> {
    if !(-MAX_TICK..=MAX_TICK).contains(&tick) {
        return Err(err!(&format!(
            "tick not in range of <{}, {}>",
            -MAX_TICK, MAX_TICK
        )));
    }

    if !(tick % tick_spacing as i32 == 0) {
        return Err(err!("tick not divisible by tick spacing"));
    }

    let bitmap_index = (tick + MAX_TICK) / tick_spacing as i32;

    let chunk: u16 = (bitmap_index / CHUNK_SIZE) as u16;
    let bit: u8 = (bitmap_index % CHUNK_SIZE) as u8;

    Ok((chunk, bit))
}

pub fn tick_to_position(tick: i32, tick_spacing: u16) -> (u16, u8) {
    assert!(
        (-MAX_TICK..=MAX_TICK).contains(&tick),
        "tick not in range of <{}, {}>",
        -MAX_TICK,
        MAX_TICK
    );

    assert_eq!(
        (tick % tick_spacing as i32),
        0,
        "tick not divisible by tick spacing"
    );

    let bitmap_index = (tick + MAX_TICK) / tick_spacing as i32;

    let chunk: u16 = (bitmap_index / CHUNK_SIZE) as u16;
    let bit: u8 = (bitmap_index % CHUNK_SIZE) as u8;

    (chunk, bit)
}
#[wasm_wrapper]
pub fn position_to_tick(chunk: u16, bit: u8, tick_spacing: u16) -> i32 {
    let tick_range_limit = MAX_TICK - MAX_TICK % tick_spacing as i32;
    (chunk as i32 * CHUNK_SIZE * tick_spacing as i32 + bit as i32 * tick_spacing as i32)
        - tick_range_limit
}

pub fn get_bit_at_position(value: u64, position: u8) -> u64 {
    (value >> position) & 1
}

fn flip_bit_at_position(value: u64, position: u8) -> u64 {
    value ^ (1 << position)
}

pub fn get_search_limit(tick: i32, tick_spacing: u16, up: bool) -> i32 {
    let index = tick / tick_spacing as i32;

    // limit unscaled
    let limit = if up {
        // search range is limited to 256 at the time ...
        let range_limit = index + TICK_SEARCH_RANGE;
        // ...also ticks for sqrt_prices over 2^64 aren't needed
        let sqrt_price_limit = MAX_TICK / tick_spacing as i32;

        range_limit.min(sqrt_price_limit)
    } else {
        let range_limit = index - TICK_SEARCH_RANGE;
        let sqrt_price_limit = -MAX_TICK / tick_spacing as i32;

        range_limit.max(sqrt_price_limit)
    };

    // scaled by tick_spacing
    limit * tick_spacing as i32
}

impl Tickmap {
    pub fn next_initialized(&self, tick: i32, tick_spacing: u16) -> Option<i32> {
        let limit = get_search_limit(tick, tick_spacing, true);

        if tick + tick_spacing as i32 > MAX_TICK {
            return None;
        }

        // add 1 to not check current tick
        let (mut chunk, mut bit) =
            tick_to_position(tick.checked_add(tick_spacing as i32)?, tick_spacing);
        let (limiting_chunk, limiting_bit) = tick_to_position(limit, tick_spacing);

        while chunk < limiting_chunk || (chunk == limiting_chunk && bit <= limiting_bit) {
            let mut shifted = self.bitmap.get(&(chunk as u64)).copied().unwrap_or(0) >> bit;

            if shifted != 0 {
                while shifted.checked_rem(2)? == 0 {
                    shifted >>= 1;
                    bit = bit.checked_add(1)?;
                }

                return if chunk < limiting_chunk || (chunk == limiting_chunk && bit <= limiting_bit)
                {
                    // no possibility of overflow
                    let index: i32 = (chunk as i32 * CHUNK_SIZE) + bit as i32;

                    Some(
                        index
                            .checked_sub(MAX_TICK / tick_spacing as i32)?
                            .checked_mul(tick_spacing.into())?,
                    )
                } else {
                    None
                };
            }

            // go to the text chunk
            // if let value = chunk.checked_add(1)? {
            if let Some(value) = chunk.checked_add(1) {
                chunk = value;
            } else {
                return None;
            }
            bit = 0;
        }

        None
    }

    // tick_spacing - spacing already scaled by tick_spacing
    pub fn prev_initialized(&self, tick: i32, tick_spacing: u16) -> Option<i32> {
        // don't subtract 1 to check the current tick
        let limit = get_search_limit(tick, tick_spacing, false); // limit scaled by tick_spacing
        let (mut chunk, mut bit) = tick_to_position(tick, tick_spacing);
        let (limiting_chunk, limiting_bit) = tick_to_position(limit, tick_spacing);

        while chunk > limiting_chunk || (chunk == limiting_chunk && bit >= limiting_bit) {
            // always safe due to limitated domain of bit variable
            let mut mask = 1u128 << bit; // left = MSB direction (increase value)
            let value = self.bitmap.get(&(chunk as u64)).copied().unwrap_or(0) as u128;

            // enter if some of previous bits are initialized in current chunk
            if value.checked_rem(mask.checked_shl(1)?)? > 0 {
                // skip uninitalized ticks
                while value & mask == 0 {
                    mask >>= 1;
                    bit = bit.checked_sub(1)?;
                }

                // return first initalized tick if limiit is not exceeded, otherswise return None
                return if chunk > limiting_chunk || (chunk == limiting_chunk && bit >= limiting_bit)
                {
                    // no possibility to overflow
                    let index: i32 = (chunk as i32 * CHUNK_SIZE) + bit as i32;

                    Some(
                        index
                            .checked_sub(MAX_TICK / tick_spacing as i32)?
                            .checked_mul(tick_spacing.into())?,
                    )
                } else {
                    None
                };
            }

            // go to the next chunk
            // if let value = chunk.checked_sub(1)? {
            if let Some(value) = chunk.checked_sub(1) {
                chunk = value;
            } else {
                return None;
            }
            bit = CHUNK_SIZE as u8 - 1;
        }

        None
    }

    // Finds closes initialized tick in direction of trade
    // and compares its sqrt_price to the sqrt_price limit of the trade
    pub fn get_closer_limit(
        &self,
        sqrt_price_limit: SqrtPrice,
        x_to_y: bool,
        current_tick: i32,
        tick_spacing: u16,
    ) -> Result<(SqrtPrice, Option<(i32, bool)>), InvariantError> {
        let closes_tick_index = if x_to_y {
            self.prev_initialized(current_tick, tick_spacing)
        } else {
            self.next_initialized(current_tick, tick_spacing)
        };

        match closes_tick_index {
            Some(index) => {
                let sqrt_price = calculate_sqrt_price(index).unwrap();

                if (x_to_y && sqrt_price > sqrt_price_limit)
                    || (!x_to_y && sqrt_price < sqrt_price_limit)
                {
                    Ok((sqrt_price, Some((index, true))))
                } else {
                    Ok((sqrt_price_limit, None))
                }
            }
            None => {
                let index = get_search_limit(current_tick, tick_spacing, !x_to_y);
                let sqrt_price = calculate_sqrt_price(index).unwrap();

                if current_tick == index {
                    return Err(InvariantError::TickLimitReached);
                }

                if (x_to_y && sqrt_price > sqrt_price_limit)
                    || (!x_to_y && sqrt_price < sqrt_price_limit)
                {
                    Ok((sqrt_price, Some((index, false))))
                } else {
                    Ok((sqrt_price_limit, None))
                }
            }
        }
    }

    pub fn get(&self, tick: i32, tick_spacing: u16) -> bool {
        let (chunk, bit) = tick_to_position(tick, tick_spacing);
        let returned_chunk = self.bitmap.get(&(chunk as u64)).copied().unwrap_or(0);
        get_bit_at_position(returned_chunk, bit) == 1
    }

    pub fn flip(&mut self, value: bool, tick: i32, tick_spacing: u16) {
        let (chunk, bit) = tick_to_position(tick, tick_spacing);
        let returned_chunk = self.bitmap.get(&(chunk as u64)).copied().unwrap_or(0);

        assert_eq!(
            get_bit_at_position(returned_chunk, bit) == 0,
            value,
            "tick initialize tick again"
        );

        self.bitmap
            .insert(chunk as u64, flip_bit_at_position(returned_chunk, bit));
    }
}
