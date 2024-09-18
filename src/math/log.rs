use decimal::*;
use traceable_result::*;

use crate::math::consts::*;
use crate::math::types::sqrt_price::SqrtPrice;

const LOG2_SCALE: u8 = 64;
const LOG2_ONE: u128 = 1 << LOG2_SCALE;
const LOG2_HALF: u128 = LOG2_ONE >> 1;
const LOG2_TWO: u128 = LOG2_ONE << 1;
const LOG2_DOUBLE_ONE: U256 = U256([0, 0, 1, 0]); // 1 << LOG2_SCALE * 2
const LOG2_SQRT_10001: u128 = 1330584781654116; // adjusted to fit the approximation of the SqrtPrice for tick = 1;
const LOG2_NEGATIVE_MAX_LOSE: u128 = 1330580000000000 * 7 / 9; // max accuracy in <-MAX_TICK, 0> domain
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

pub fn get_tick_at_sqrt_price(sqrt_price: SqrtPrice, tick_spacing: u16) -> TrackableResult<i32> {
    if sqrt_price.get() > MAX_SQRT_PRICE || sqrt_price.get() < MIN_SQRT_PRICE {
        return Err(err!("sqrt_price out of range"));
    }

    let sqrt_price_x64: u128 = sqrt_price_to_x64(sqrt_price);
    let (log2_sign, log2_sqrt_price) = log2_iterative_approximation_x64(sqrt_price_x64);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt_price_to_u64() {
        // min sqrt_price -> sqrt(1.0001)^MIN_TICK
        {
            let min_sqrt_price_decimal = SqrtPrice::from_tick(-MAX_TICK).unwrap();

            let min_sqrt_price_x64 = sqrt_price_to_x64(min_sqrt_price_decimal);

            let expected_min_sqrt_price_x64 = 65534;
            let nearly_min_sqrt_price_decimal =
                sqrt_price_to_x64(SqrtPrice::from_tick(-MAX_TICK + 1).unwrap());
            let nearly_min_sqrt_price_decimal2 =
                sqrt_price_to_x64(SqrtPrice::from_tick(-MAX_TICK + 2).unwrap());

            assert_ne!(
                nearly_min_sqrt_price_decimal2,
                nearly_min_sqrt_price_decimal
            );
            assert_ne!(nearly_min_sqrt_price_decimal, expected_min_sqrt_price_x64);
            assert_ne!(
                sqrt_price_to_x64(SqrtPrice::from_tick(-2).unwrap()),
                sqrt_price_to_x64(SqrtPrice::from_tick(-1).unwrap())
            );

            assert_eq!(min_sqrt_price_x64, expected_min_sqrt_price_x64);
        }
        // max sqrt_price -> sqrt(1.0001)^MAX_TICK
        {
            let max_sqrt_price_decimal = SqrtPrice::from_tick(MAX_TICK).unwrap();
            let max_sqrt_price_x64 = sqrt_price_to_x64(max_sqrt_price_decimal);

            let expected_max_sqrt_price_x64 = 5192410085712699832897385122752987;
            assert_eq!(max_sqrt_price_x64, expected_max_sqrt_price_x64);
        }
    }

    #[test]
    fn test_log2_x64() {
        // log2 of 1
        {
            let sqrt_price_decimal = SqrtPrice::from_integer(1);
            let sqrt_price_x64 = sqrt_price_to_x64(sqrt_price_decimal);
            let (sign, value) = log2_iterative_approximation_x64(sqrt_price_x64);
            assert!(sign);
            assert_eq!(value, 0);
        }
        // log2 > 0 when x > 1
        {
            let sqrt_price_decimal = SqrtPrice::from_integer(879);
            let sqrt_price_x64 = sqrt_price_to_x64(sqrt_price_decimal);

            let (sign, value) = log2_iterative_approximation_x64(sqrt_price_x64);
            assert!(sign);
            assert_eq!(value, 180403980057034096640);
        }
        // log2 < 0 when x < 1
        {
            let sqrt_price_decimal = SqrtPrice::from_scale(59, 4);
            let sqrt_price_x64 = sqrt_price_to_x64(sqrt_price_decimal);
            let (sign, value) = log2_iterative_approximation_x64(sqrt_price_x64);
            assert!(!sign);
            assert_eq!(value, 136599418782046486528);
        }
        // log2 of max sqrt_price
        {
            let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
            let sqrt_price_x64 = sqrt_price_to_x64(max_sqrt_price);
            let (sign, value) = log2_iterative_approximation_x64(sqrt_price_x64);
            assert!(sign);
            assert_eq!(value, 885444295875638853632);
        }
        // log2 of min sqrt_price
        {
            let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
            let sqrt_price_x64 = sqrt_price_to_x64(min_sqrt_price);
            let (sign, value) = log2_iterative_approximation_x64(sqrt_price_x64);
            assert!(!sign);
            assert_eq!(value, 885444121623709614080);
        }
        // log2 of sqrt(1.0001^(-19_999)) - 1
        {
            let mut sqrt_price_decimal = SqrtPrice::from_tick(-19_999).unwrap();
            sqrt_price_decimal -= SqrtPrice::new(1);
            let sqrt_price_x64 = sqrt_price_to_x64(sqrt_price_decimal);
            let (sign, value) = log2_iterative_approximation_x64(sqrt_price_x64);
            assert!(!sign);
            assert_eq!(value, 26610365048300503040);
        }
        // log2 of sqrt(1.0001^(19_999)) + 1
        {
            let mut sqrt_price_decimal = SqrtPrice::from_tick(19_999).unwrap();
            sqrt_price_decimal -= SqrtPrice::new(1);
            let sqrt_price_x64 = sqrt_price_to_x64(sqrt_price_decimal);
            let (sign, value) = log2_iterative_approximation_x64(sqrt_price_x64);
            assert!(sign);
            assert_eq!(value, 26610365048300503040);
        }
    }

    #[test]
    fn test_get_tick_at_sqrt_price_x64() {
        // around 0 tick
        {
            // get tick at 1
            {
                let sqrt_price_decimal = SqrtPrice::from_integer(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), 0);
            }
            // get tick slightly below 1
            {
                let sqrt_price_decimal = SqrtPrice::from_integer(1) - SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), -1);
            }
            // get tick slightly above 1
            {
                let sqrt_price_decimal = SqrtPrice::from_integer(1) + SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), 0);
            }
        }
        // around 1 tick
        {
            let sqrt_price_decimal = SqrtPrice::from_tick(1).unwrap();
            // get tick at sqrt(1.0001)
            {
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), 1);
            }
            // get tick slightly below sqrt(1.0001)
            {
                let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), 0);
            }
            // get tick slightly above sqrt(1.0001)
            {
                let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), 1);
            }
        }
        // around -1 tick
        {
            let sqrt_price_decimal = SqrtPrice::from_tick(-1).unwrap();
            // get tick at sqrt(1.0001^(-1))
            {
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), -1);
            }
            // get tick slightly below sqrt(1.0001^(-1))
            {
                let sqrt_price_decimal = SqrtPrice::from_tick(-1).unwrap() - SqrtPrice::new(1);

                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                let tick = tick.unwrap();
                assert_eq!(tick, -2);
            }
            // get tick slightly above sqrt(1.0001^(-1))
            {
                let sqrt_price_decimal = SqrtPrice::from_tick(-1).unwrap() + SqrtPrice::new(3);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), -1);
            }
        }
        // around max - 1 tick
        {
            let sqrt_price_decimal = SqrtPrice::from_tick(MAX_TICK).unwrap();
            let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
            assert_eq!(tick.unwrap(), MAX_TICK);

            let sqrt_price_decimal = SqrtPrice::from_tick(MAX_TICK - 1).unwrap();
            // get tick at sqrt(1.0001^(MAX_TICK - 1))
            {
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), MAX_TICK - 1);
            }
            // get tick slightly below sqrt(1.0001^(MAX_TICK - 1))
            {
                let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), MAX_TICK - 2);
            }
            // get tick slightly above sqrt(1.0001^(MAX_TICK - 1))
            {
                let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), MAX_TICK - 1);
            }
        }
        // around min + 1 tick
        {
            let sqrt_price_decimal = SqrtPrice::from_tick(-(MAX_TICK - 1)).unwrap();
            // get tick at sqrt(1.0001^(-MAX_TICK + 1))
            {
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), -(MAX_TICK - 1));
            }
            // get tick slightly below sqrt(1.0001^(-MAX_TICK + 1))
            {
                let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), -MAX_TICK);
            }
            // get tick slightly above sqrt(1.0001^(-MAX_TICK + 1))
            {
                let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), -(MAX_TICK - 1));
            }
        }
        //get tick slightly below at max tick
        {
            let max_sqrt_price = SqrtPrice::new(MAX_SQRT_PRICE);
            let sqrt_price_decimal = max_sqrt_price - SqrtPrice::new(1);
            let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
            assert_eq!(tick.unwrap(), MAX_TICK - 1);
        }
        // around 19_999 tick
        {
            let expected_tick = 19_999;
            let sqrt_price_decimal = SqrtPrice::from_tick(expected_tick).unwrap();
            // get tick at sqrt(1.0001^19_999)
            {
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), expected_tick);
            }
            // get tick slightly below sqrt(1.0001^19_999)
            {
                let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);

                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), expected_tick - 1);
            }
            // get tick slightly above sqrt(1.0001^19_999)
            {
                let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), expected_tick);
            }
        }
        // around -19_999 tick
        {
            let expected_tick = -19_999;
            let sqrt_price_decimal = SqrtPrice::from_tick(expected_tick).unwrap();
            // get tick at sqrt(1.0001^(-19_999))
            {
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), expected_tick);
            }
            // get tick slightly below sqrt(1.0001^(-19_999))
            {
                // let sqrt_price_decimal = sqrt_price_decimal - Decimal::new(150);
                let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), expected_tick - 1);
            }
            // get tick slightly above sqrt(1.0001^(-19_999))
            {
                let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                assert_eq!(tick.unwrap(), expected_tick);
            }
        }
        //get tick slightly above at min tick
        {
            let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
            let sqrt_price_decimal = min_sqrt_price + SqrtPrice::new(1);
            let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
            assert_eq!(tick.unwrap(), -MAX_TICK);
        }
    }

    #[test]
    fn test_domain_calculate_sqrt_price() {
        // Over max tick
        {
            let tick_out_of_range = MAX_TICK + 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
        // Below min tick
        {
            let tick_out_of_range = -MAX_TICK - 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
    }

    #[test]
    fn test_align_tick_with_spacing() {
        // zero
        {
            let accurate_tick = 0;
            let tick_spacing = 3;

            let tick_with_spacing = align_tick_to_spacing(accurate_tick, tick_spacing);
            assert_eq!(tick_with_spacing, 0);
        }
        // positive
        {
            let accurate_tick = 14;
            let tick_spacing = 10;

            let tick_with_spacing = align_tick_to_spacing(accurate_tick, tick_spacing);
            assert_eq!(tick_with_spacing, 10);
        }
        // positive at tick
        {
            let accurate_tick = 20;
            let tick_spacing = 10;

            let tick_with_spacing = align_tick_to_spacing(accurate_tick, tick_spacing);
            assert_eq!(tick_with_spacing, 20);
        }
        // negative
        {
            let accurate_tick = -14;
            let tick_spacing = 10;

            let tick_with_spacing = align_tick_to_spacing(accurate_tick, tick_spacing);
            assert_eq!(tick_with_spacing, -20);
        }
        // negative at tick
        {
            let accurate_tick = -120;
            let tick_spacing = 3;

            let tick_with_spacing = align_tick_to_spacing(accurate_tick, tick_spacing);
            assert_eq!(tick_with_spacing, -120);
        }
    }

    #[test]
    fn test_all_positive_ticks() {
        for n in 0..MAX_TICK {
            {
                let expected_tick = n;
                let sqrt_price_decimal = SqrtPrice::from_tick(expected_tick).unwrap();
                // get tick at sqrt(1.0001^(n))
                {
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
                // get tick slightly below sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                    assert_eq!(tick.unwrap(), expected_tick - 1);
                }
                // get tick slightly above sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
            }
        }
    }

    #[test]
    fn test_all_negative_ticks() {
        for n in 0..MAX_TICK {
            {
                let expected_tick = -n;
                let sqrt_price_decimal = SqrtPrice::from_tick(expected_tick).unwrap();
                // get tick at sqrt(1.0001^(n))
                {
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
                // get tick slightly below sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);
                    assert_eq!(tick.unwrap(), expected_tick - 1);
                }
                // get tick slightly above sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, 1);

                    assert_eq!(tick.unwrap(), expected_tick);
                }
            }
        }
    }

    #[test]
    fn test_all_positive_tick_spacing_greater_than_1() {
        let tick_spacing: i32 = 3;
        for n in 0..MAX_TICK {
            {
                let input_tick = n;
                let sqrt_price_decimal = SqrtPrice::from_tick(input_tick).unwrap();
                // get tick at sqrt(1.0001^(n))
                {
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, tick_spacing as u16);
                    let expected_tick = align_tick_to_spacing(input_tick, tick_spacing);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
                // get tick slightly below sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, tick_spacing as u16);
                    let expected_tick = align_tick_to_spacing(input_tick - 1, tick_spacing);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
                // get tick slightly above sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, tick_spacing as u16);
                    let expected_tick = align_tick_to_spacing(input_tick, tick_spacing);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
            }
        }
    }

    #[test]
    fn test_all_negative_tick_spacing_greater_than_1() {
        let tick_spacing: i32 = 4;
        for n in 0..MAX_TICK {
            {
                let input_tick = -n;
                let sqrt_price_decimal = SqrtPrice::from_tick(input_tick).unwrap();
                // get tick at sqrt(1.0001^(n))
                {
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, tick_spacing as u16);
                    let expected_tick = align_tick_to_spacing(input_tick, tick_spacing);

                    assert_eq!(tick.unwrap(), expected_tick);
                }
                // get tick slightly below sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal - SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, tick_spacing as u16);
                    let expected_tick = align_tick_to_spacing(input_tick - 1, tick_spacing);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
                // get tick slightly above sqrt(1.0001^n)
                {
                    let sqrt_price_decimal = sqrt_price_decimal + SqrtPrice::new(1);
                    let tick = get_tick_at_sqrt_price(sqrt_price_decimal, tick_spacing as u16);
                    let expected_tick = align_tick_to_spacing(input_tick, tick_spacing);
                    assert_eq!(tick.unwrap(), expected_tick);
                }
            }
        }
    }
}
