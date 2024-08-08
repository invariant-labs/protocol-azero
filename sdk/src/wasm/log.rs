use crate::consts::*;
use crate::types::sqrt_price::SqrtPrice;
use decimal::*;
use js_sys::BigInt;
use traceable_result::*;
use wasm_bindgen::prelude::*;
use wasm_wrapper::*;
use core::convert::TryInto;

const LOG2_SCALE: u8 = 64;
const LOG2_ONE: u128 = 1 << LOG2_SCALE;
const LOG2_HALF: u128 = LOG2_ONE >> 1;
const LOG2_TWO: u128 = LOG2_ONE << 1;
const LOG2_DOUBLE_ONE: U256 = U256([0, 0, 1, 0]); // 1 << LOG2_SCALE * 2
const LOG2_SQRT_10001: u64 = 1330584781654116; // adjusted to fit the approximation of the SqrtPrice for tick = 1;
const LOG2_NEGATIVE_MAX_LOSE: u64 = 1330580000000000 * 7 / 9; // max accuracy in <-MAX_TICK, 0> domain
const LOG2_MIN_BINARY_POSITION: i32 = 46; // accuracy = 2^(-46)
const LOG2_ACCURACY: u64 = 1u64 << (63 - LOG2_MIN_BINARY_POSITION);
const SQRT_PRICE_DENOMINATOR: u128 = 1_000000_000000_000000_000000;

fn sqrt_price_to_x64(decimal: SqrtPrice) -> u128 {
    u128::uint_cast(
        decimal
            .cast::<U256>()
            .checked_mul(LOG2_ONE.into())
            .unwrap()
            .checked_div(SQRT_PRICE_DENOMINATOR.into())
            .unwrap(),
    )
}

#[wasm_wrapper]
fn align_tick_to_spacing(accurate_tick: i32, tick_spacing: i32) -> i32 {
    match accurate_tick > 0 {
        true => accurate_tick
            .checked_sub(accurate_tick.checked_rem(tick_spacing).unwrap())
            .unwrap(),
        false => accurate_tick
            .checked_sub(accurate_tick.rem_euclid(tick_spacing))
            .unwrap(),
    }
}

fn log2_floor_x64(mut sqrt_price_x64: u128) -> u128 {
    let mut msb = 0;

    if sqrt_price_x64 >= 1u128 << 64 {
        sqrt_price_x64 >>= 64;
        msb |= 64;
    };
    if sqrt_price_x64 >= 1u128 << 32 {
        sqrt_price_x64 >>= 32;
        msb |= 32;
    };
    if sqrt_price_x64 >= 1u128 << 16 {
        sqrt_price_x64 >>= 16;
        msb |= 16;
    };
    if sqrt_price_x64 >= 1u128 << 8 {
        sqrt_price_x64 >>= 8;
        msb |= 8;
    };
    if sqrt_price_x64 >= 1u128 << 4 {
        sqrt_price_x64 >>= 4;
        msb |= 4;
    };
    if sqrt_price_x64 >= 1u128 << 2 {
        sqrt_price_x64 >>= 2;
        msb |= 2;
    };
    if sqrt_price_x64 >= 1u128 << 1 {
        msb |= 1;
    };

    msb
}

fn log2_iterative_approximation_x64(mut sqrt_price_x64: u128) -> (bool, u128) {
    let mut sign = true;
    // log2(x) = -log2(1/x), when x < 1

    if (sqrt_price_x64) < LOG2_ONE {
        sign = false;
        sqrt_price_x64 = (LOG2_DOUBLE_ONE
            .checked_div(U256::from(sqrt_price_x64).checked_add(1.into()).unwrap()))
        .unwrap()
        .try_into()
        .unwrap()
    }
    let log2_floor = log2_floor_x64(sqrt_price_x64 >> LOG2_SCALE);
    let mut result = log2_floor << LOG2_SCALE;
    let mut y: u128 = sqrt_price_x64 >> log2_floor;

    if y == LOG2_ONE {
        return (sign, result);
    };
    let mut delta: u128 = LOG2_HALF;
    while delta > LOG2_ACCURACY as u128 {
        y = u128::uint_cast(
            U256::from(y)
                .checked_mul(U256::from(y))
                .unwrap()
                .checked_div(LOG2_ONE.into())
                .unwrap(),
        );
        if y >= LOG2_TWO {
            result |= delta;
            y >>= 1;
        }
        delta >>= 1;
    }
    (sign, result)
}

#[wasm_wrapper("calculateTick")]
pub fn get_tick_at_sqrt_price(sqrt_price: SqrtPrice, tick_spacing: u16) -> TrackableResult<i32> {
    if sqrt_price.get() > MAX_SQRT_PRICE || sqrt_price.get() < MIN_SQRT_PRICE {
        return Err(err!("sqrt_price out of range"));
    }

    let sqrt_price_x64: u128 = sqrt_price_to_x64(sqrt_price);
    let (log2_sign, log2_sqrt_price) = log2_iterative_approximation_x64(sqrt_price_x64);

    let abs_floor_tick: i32 = match log2_sign {
        true => log2_sqrt_price
            .checked_div(LOG2_SQRT_10001 as u128)
            .unwrap(),
        false => (log2_sqrt_price
            .checked_add(LOG2_NEGATIVE_MAX_LOSE as u128)
            .unwrap())
        .checked_div(LOG2_SQRT_10001 as u128)
        .unwrap(),
    } as i32;

    let nearer_tick = match log2_sign {
        true => abs_floor_tick,
        false => 0i32.checked_sub(abs_floor_tick).unwrap(),
    };

    let farther_tick = match log2_sign {
        true => abs_floor_tick.checked_add(1).unwrap(),
        false => 0i32
            .checked_sub(abs_floor_tick)
            .unwrap()
            .checked_sub(1)
            .unwrap(),
    };
    let farther_tick_with_spacing = align_tick_to_spacing(farther_tick, tick_spacing as i32);
    let nearer_tick_with_spacing = align_tick_to_spacing(nearer_tick, tick_spacing as i32);
    if farther_tick_with_spacing == nearer_tick_with_spacing {
        return Ok(nearer_tick_with_spacing);
    };

    let accurate_tick = match log2_sign {
        true => {
            let farther_tick_sqrt_price_decimal =
                ok_or_mark_trace!(SqrtPrice::from_tick(farther_tick))?;
            match sqrt_price >= farther_tick_sqrt_price_decimal {
                true => farther_tick_with_spacing,
                false => nearer_tick_with_spacing,
            }
        }
        false => {
            let nearer_tick_sqrt_price_decimal =
                ok_or_mark_trace!(SqrtPrice::from_tick(nearer_tick))?;
            match nearer_tick_sqrt_price_decimal <= sqrt_price {
                true => nearer_tick_with_spacing,
                false => farther_tick_with_spacing,
            }
        }
    };
    Ok(match tick_spacing > 1 {
        true => align_tick_to_spacing(accurate_tick, tick_spacing as i32),
        false => accurate_tick,
    })
}
