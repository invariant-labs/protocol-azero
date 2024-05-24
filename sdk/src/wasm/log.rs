use crate::consts::*;
use crate::types::sqrt_price::SqrtPrice;
use decimal::*;
use traceable_result::*;

const LOG2_SCALE: u8 = 32;
const LOG2_DOUBLE_SCALE: u8 = 64;
const LOG2_ONE: u128 = 1 << LOG2_SCALE;
const LOG2_HALF: u64 = (LOG2_ONE >> 1) as u64;
const LOG2_TWO: u128 = LOG2_ONE << 1;
const LOG2_DOUBLE_ONE: u128 = 1 << LOG2_DOUBLE_SCALE;
const LOG2_SQRT_10001: u64 = 309801;
const LOG2_NEGATIVE_MAX_LOSE: u64 = 300000; // max accuracy in <-MAX_TICK, 0> domain
const LOG2_MIN_BINARY_POSITION: i32 = 15; // accuracy = 2^(-15)
const LOG2_ACCURACY: u64 = 1u64 << (31 - LOG2_MIN_BINARY_POSITION);
const SQRT_PRICE_DENOMINATOR: u128 = 1_000000_000000_000000_000000;

fn sqrt_price_to_x32(decimal: SqrtPrice) -> u64 {
    (decimal
        .get()
        .checked_mul(LOG2_ONE)
        .unwrap()
        .checked_div(SQRT_PRICE_DENOMINATOR)
        .unwrap()) as u64
}

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

fn log2_floor_x32(mut sqrt_price_x32: u64) -> u64 {
    let mut msb = 0;

    if sqrt_price_x32 >= 1u64 << 32 {
        sqrt_price_x32 >>= 32;
        msb |= 32;
    };
    if sqrt_price_x32 >= 1u64 << 16 {
        sqrt_price_x32 >>= 16;
        msb |= 16;
    };
    if sqrt_price_x32 >= 1u64 << 8 {
        sqrt_price_x32 >>= 8;
        msb |= 8;
    };
    if sqrt_price_x32 >= 1u64 << 4 {
        sqrt_price_x32 >>= 4;
        msb |= 4;
    };
    if sqrt_price_x32 >= 1u64 << 2 {
        sqrt_price_x32 >>= 2;
        msb |= 2;
    };
    if sqrt_price_x32 >= 1u64 << 1 {
        msb |= 1;
    };

    msb
}

fn log2_iterative_approximation_x32(mut sqrt_price_x32: u64) -> (bool, u64) {
    let mut sign = true;
    // log2(x) = -log2(1/x), when x < 1
    if (sqrt_price_x32 as u128) < LOG2_ONE {
        sign = false;
        sqrt_price_x32 = (LOG2_DOUBLE_ONE
            .checked_div((sqrt_price_x32 as u128).checked_add(1).unwrap()))
        .unwrap() as u64
    }
    let log2_floor = log2_floor_x32(sqrt_price_x32 >> LOG2_SCALE);
    let mut result = log2_floor << LOG2_SCALE;
    let mut y: u128 = (sqrt_price_x32 as u128) >> log2_floor;

    if y == LOG2_ONE {
        return (sign, result);
    };
    let mut delta: u64 = LOG2_HALF;
    while delta > LOG2_ACCURACY {
        y = y.checked_mul(y).unwrap() / LOG2_ONE;
        if y >= LOG2_TWO {
            result |= delta;
            y >>= 1;
        }
        delta >>= 1;
    }
    (sign, result)
}

pub fn get_tick_at_sqrt_price(sqrt_price: SqrtPrice, tick_spacing: u16) -> TrackableResult<i32> {
    if sqrt_price.get() > MAX_SQRT_PRICE || sqrt_price.get() < MIN_SQRT_PRICE {
        return Err(err!("sqrt_price out of range"));
    }

    let sqrt_price_x32: u64 = sqrt_price_to_x32(sqrt_price);
    let (log2_sign, log2_sqrt_price) = log2_iterative_approximation_x32(sqrt_price_x32);

    let abs_floor_tick: i32 = match log2_sign {
        true => log2_sqrt_price.checked_div(LOG2_SQRT_10001).unwrap(),
        false => (log2_sqrt_price.checked_add(LOG2_NEGATIVE_MAX_LOSE).unwrap())
            .checked_div(LOG2_SQRT_10001)
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
