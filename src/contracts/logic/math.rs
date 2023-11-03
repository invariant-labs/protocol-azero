use decimal::*;
use traceable_result::*;

use crate::math::liquidity::Liquidity;
use crate::math::sqrt_price::sqrt_price::{calculate_sqrt_price, SqrtPrice};
use crate::math::token_amount::TokenAmount;
// use crate::math::MAX_TICK;

#[derive(Debug)]
pub struct LiquidityResult {
    pub x: TokenAmount,
    pub y: TokenAmount,
    pub l: Liquidity,
}

#[derive(Debug)]
pub struct SingleTokenLiquidity {
    pub l: Liquidity,
    pub amount: TokenAmount,
}

pub fn get_liquidity(
    x: TokenAmount,
    y: TokenAmount,
    lower_tick: i32,
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
    tick_spacing: u16,
) -> TrackableResult<LiquidityResult> {
    // if lower_tick < -MAX_TICK {
    //     lower_tick = get_min_tick(tick_spacing);
    // }
    // if upper_tick > MAX_TICK {
    //     upper_tick = get_max_tick(tick_spacing);
    // }

    let lower_sqrt_price = calculate_sqrt_price(lower_tick).unwrap();
    let upper_sqrt_price = calculate_sqrt_price(upper_tick).unwrap();

    if upper_sqrt_price < current_sqrt_price {
        // single token y
        let result_by_y = ok_or_mark_trace!(get_liquidity_by_y_sqrt_price(
            y,
            lower_sqrt_price,
            upper_sqrt_price,
            current_sqrt_price,
            rounding_up,
        ))?;
        return Ok(LiquidityResult {
            x: result_by_y.amount,
            y,
            l: result_by_y.l,
        });
    } else if current_sqrt_price < lower_sqrt_price {
        // single token x
        let result_by_x = ok_or_mark_trace!(get_liquidity_by_x_sqrt_price(
            x,
            lower_sqrt_price,
            upper_sqrt_price,
            current_sqrt_price,
            rounding_up,
        ))?;
        return Ok(LiquidityResult {
            x,
            y: result_by_x.amount,
            l: result_by_x.l,
        });
    }
    let result_by_y = ok_or_mark_trace!(get_liquidity_by_y_sqrt_price(
        y,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))?;
    let result_by_x = ok_or_mark_trace!(get_liquidity_by_x_sqrt_price(
        x,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))?;
    Ok(if result_by_y.l < result_by_x.l {
        LiquidityResult {
            x: result_by_y.amount,
            y: result_by_x.amount,
            l: result_by_y.l,
        }
    } else {
        LiquidityResult {
            x: result_by_x.amount,
            y: result_by_y.amount,
            l: result_by_x.l,
        }
    })
}

pub fn get_liquidity_by_x(
    x: TokenAmount,
    lower_tick: i32,
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
    tick_spacing: u16,
) -> TrackableResult<SingleTokenLiquidity> {
    // if lower_tick < -MAX_TICK {
    //     lower_tick = get_min_tick(tick_spacing);
    // }
    // if upper_tick > MAX_TICK {
    //     upper_tick = get_max_tick(tick_spacing);
    // }

    let lower_sqrt_price = calculate_sqrt_price(lower_tick).unwrap();
    let upper_sqrt_price = calculate_sqrt_price(upper_tick).unwrap();

    ok_or_mark_trace!(get_liquidity_by_x_sqrt_price(
        x,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))
}
pub fn get_liquidity_by_x_sqrt_price(
    x: TokenAmount,
    lower_sqrt_price: SqrtPrice,
    upper_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> TrackableResult<SingleTokenLiquidity> {
    if upper_sqrt_price < current_sqrt_price {
        return Err(err!("Wrong token"));
    }

    if current_sqrt_price < lower_sqrt_price {
        // Checked_from_decimal_to_value
        let nominator =
            (lower_sqrt_price.big_mul(upper_sqrt_price)).big_div(SqrtPrice::from_integer(1));
        let denominator = upper_sqrt_price - lower_sqrt_price;
        let liquidity = Liquidity::from_integer((x.0 * nominator.get()) / denominator.get());
        return Ok(SingleTokenLiquidity {
            l: liquidity,
            amount: TokenAmount(0),
        });
        // return (liquidity, TokenAmount(0));
    }

    let nominator = current_sqrt_price
        .big_mul(upper_sqrt_price)
        .big_div(SqrtPrice::from_integer(1));
    let denominator = upper_sqrt_price - current_sqrt_price;
    let liquidity = Liquidity::from_integer((x.0 * nominator.get()) / denominator.get());
    let sqrt_price_diff = current_sqrt_price - lower_sqrt_price;
    let y = ok_or_mark_trace!(calculate_y(sqrt_price_diff, liquidity, rounding_up))?;
    Ok(SingleTokenLiquidity {
        l: liquidity,
        amount: y,
    })
}

pub fn get_liquidity_by_y(
    y: TokenAmount,
    lower_tick: i32,
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
    tick_spacing: u16,
) -> TrackableResult<SingleTokenLiquidity> {
    // if lower_tick < -MAX_TICK {
    //     lower_tick = get_min_tick(tick_spacing);
    // }
    // if upper_tick > MAX_TICK {
    //     upper_tick = get_max_tick(tick_spacing);
    // }

    let lower_sqrt_price = calculate_sqrt_price(lower_tick).unwrap();
    let upper_sqrt_price = calculate_sqrt_price(upper_tick).unwrap();

    ok_or_mark_trace!(get_liquidity_by_y_sqrt_price(
        y,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))
}

pub fn get_liquidity_by_y_sqrt_price(
    y: TokenAmount,
    lower_sqrt_price: SqrtPrice,
    upper_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> TrackableResult<SingleTokenLiquidity> {
    // if current_sqrt_price < lower_sqrt_price {
    //     return Err(err!(TrackableError::cast::<TokenAmount>().as_str()));
    // }

    if upper_sqrt_price <= current_sqrt_price {
        let sqrt_price_diff = upper_sqrt_price - lower_sqrt_price;
        let liquidity =
            Liquidity::from_integer(y.0 * SqrtPrice::from_integer(1).get() / sqrt_price_diff.get());
        return Ok(SingleTokenLiquidity {
            l: liquidity,
            amount: TokenAmount::new(0),
        });
    }

    let sqrt_price_diff = current_sqrt_price - lower_sqrt_price;
    let liquidity =
        Liquidity::from_integer(y.0 * SqrtPrice::from_integer(1).get() / sqrt_price_diff.get());
    let denominator =
        (current_sqrt_price.big_mul(upper_sqrt_price)).big_div(SqrtPrice::from_integer(1));
    let nominator = upper_sqrt_price - current_sqrt_price;

    let x = ok_or_mark_trace!(calculate_x(nominator, denominator, liquidity, rounding_up))?;

    Ok(SingleTokenLiquidity {
        l: liquidity,
        amount: x,
    })
}

pub fn calculate_x(
    nominator: SqrtPrice,
    denominator: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let common = liquidity.big_mul(nominator).big_div(denominator).get();

    if rounding_up {
        Ok(TokenAmount::new(
            ((common + Liquidity::from_integer(1).get()) - 1) / Liquidity::from_integer(1).get(),
        ))
    } else {
        Ok(TokenAmount::new(common / Liquidity::from_integer(1).get()))
    }
}

pub fn calculate_y(
    sqrt_price_diff: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let shifted_liquidity = liquidity.get() / Liquidity::from_integer(1).get();
    if rounding_up {
        Ok(TokenAmount::new(
            ((sqrt_price_diff.get() * shifted_liquidity) + (SqrtPrice::from_integer(1).get() - 1))
                / SqrtPrice::from_integer(1).get(),
        ))
    } else {
        Ok(TokenAmount::new(
            sqrt_price_diff.get() * shifted_liquidity / SqrtPrice::from_integer(1).get(),
        ))
    }
}

// pub fn get_min_tick(tick_spacing: u16) -> i32 {
//     let limited_by_price = -MAX_TICK + (-MAX_TICK % tick_spacing as i32);
//     let limited_by_tickmap = -TICK_LIMIT * tick_spacing as i32 - tick_spacing as i32;
//     if limited_by_price > limited_by_tickmap {
//         limited_by_price
//     } else {
//         limited_by_tickmap
//     }
// }

// pub fn get_max_tick(tick_spacing: u16) -> i32 {
//     let limited_by_price = MAX_TICK + (MAX_TICK % tick_spacing as i32);
//     let limited_by_tickmap = TICK_LIMIT * tick_spacing as i32 - tick_spacing as i32;
//     if limited_by_price > limited_by_tickmap {
//         limited_by_price
//     } else {
//         limited_by_tickmap
//     }
// }
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_liquidity_by_x_test() {
        let x = TokenAmount::new(43_0000);
        let current_sqrt_price = calculate_sqrt_price(100).unwrap();
        // below current tick
        {
            let lower_tick = -50;
            let upper_tick = 10;
            let (_, cause, stack) =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true, 1)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "Wrong token");
            assert_eq!(stack.len(), 2);
        }
        // in current tick
        {
            let expected_l = Liquidity::new(432392997000000);
            let expected_y_up = TokenAmount(434322);
            let expected_y_down = TokenAmount(434321);

            let lower_tick = 80;
            let upper_tick = 120;

            let result_up =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true, 1).unwrap();

            let result_down =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, false, 1)
                    .unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_y_up, result_up.amount);
            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_y_down, result_down.amount);
        }
        // above current tick
        {
            let expected_l = Liquidity::new(13548826000000);
            let expected_y = TokenAmount(0);
            let lower_tick = 150;
            let upper_tick = 800;

            let result_up =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true, 1).unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_y, result_up.amount);

            let result_down =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, false, 1)
                    .unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_y, result_up.amount);
        }
    }

    #[test]
    fn get_liquidity_by_y_test() {
        let y = TokenAmount(476_000_000_00);
        let current_sqrt_price = calculate_sqrt_price(-20000).unwrap();
        // below current tick
        {
            let expected_l = Liquidity::new(2789052279103000000); // PROTOCOL = 2789052279103923275
            let expected_x = TokenAmount(0);
            let lower_tick = -22000;
            let upper_tick = -21000;

            let result_up =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true, 1).unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_x, result_up.amount);
            let result_down =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false, 1)
                    .unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_x, result_down.amount);
        }
        // in current tick
        {
            let expected_l = Liquidity::new(584945290554000000); // PROTOCOL = 584945290554346935
            let expected_x_up = TokenAmount(77539808126);
            let expected_x_down = TokenAmount(77539808125);
            let lower_tick = -25000;
            let upper_tick = -19000;

            let result_up =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true, 1).unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_x_up, result_up.amount);
            let result_down =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false, 1)
                    .unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_x_down, result_down.amount);
        }
        // above current tick
        {
            let lower_tick = -10000;
            let upper_tick = 0;

            let (_, cause, stack) =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true, 1)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "Wrong token");
            assert_eq!(stack.len(), 2);
            let (_, cause, stack) =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false, 1)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "Wrong token");
            assert_eq!(stack.len(), 2);
        }
    }

    #[test]
    fn get_liquidity_test() {
        let y = TokenAmount(476_000_000_00);
        let current_sqrt_price = calculate_sqrt_price(-20000).unwrap();

        // below current tick
        {
            let lower_tick = -22000;
            let upper_tick = -21000;
            let expected_x = TokenAmount(0);
            let expected_l = Liquidity::new(2789052279103000000); // 2789052279103923275
            let result_up = get_liquidity(
                expected_x,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            )
            .unwrap();
            let result_down = get_liquidity(
                expected_x,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            )
            .unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_l, result_down.l);
            assert_eq!(result_up.x, expected_x);
            assert_eq!(result_down.x, expected_x);
        }
        // in current tick
        {
            let lower_tick = -25000;
            let upper_tick = -19000;
            let expected_x_up = TokenAmount(77539808126);
            let expected_x_down = TokenAmount(77539808126); // 77539808125
            let expected_l_up = Liquidity::new(584945290554000000); // 584945290554346935
            let expected_l_down = Liquidity::new(584945290554000000);
            let result_up = get_liquidity(
                expected_x_up,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            )
            .unwrap();
            let result_down = get_liquidity(
                expected_x_down,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            )
            .unwrap();
            assert_eq!(expected_l_up, result_up.l);
            assert_eq!(expected_l_down, result_down.l);
            assert_eq!(result_up.x, expected_x_up);
            assert_eq!(result_down.x, expected_x_down);
        }
        // above current tick
        {
            let lower_tick = 150;
            let upper_tick = 800;
            let x = TokenAmount(43_000_000_0);
            let expected_y = TokenAmount(0);
            let expected_l = Liquidity::new(13548826311000000); // 13548826311623850
            let result_up = get_liquidity(
                x,
                expected_y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            )
            .unwrap();
            let result_down = get_liquidity(
                x,
                expected_y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            )
            .unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_l, result_down.l);
            assert_eq!(result_up.y, expected_y);
            assert_eq!(result_down.y, expected_y);
        }
    }
}