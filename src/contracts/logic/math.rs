use decimal::*;
use decimal::{BigOps, CheckedOps, Factories};

use crate::math::liquidity::Liquidity;
use crate::math::sqrt_price::sqrt_price::{calculate_sqrt_price, SqrtPrice};
use crate::math::token_amount::TokenAmount;
use crate::math::{sqrt_price, MAX_TICK};

const PRICE_DENOMINATOR: u128 = 1_000_000_000_000_000_000_000_000;
const LIQUIDITY_DENOMINATOR: u128 = 1_000_000;

// taken from protocol
const TICK_LIMIT: i32 = 44_364;

pub fn get_liquidity(
    x: TokenAmount,
    y: TokenAmount,
    mut lower_tick: i32,
    mut upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
    tick_spacing: u16,
) -> (TokenAmount, TokenAmount, Liquidity) {
    if lower_tick < -MAX_TICK {
        lower_tick = get_min_tick(tick_spacing);
    }
    if upper_tick > MAX_TICK {
        upper_tick = get_max_tick(tick_spacing);
    }

    let lower_sqrt_price = calculate_sqrt_price(lower_tick).unwrap();
    let upper_sqrt_price = calculate_sqrt_price(upper_tick).unwrap();

    if upper_sqrt_price < current_sqrt_price {
        // single token y
        let (liquidity, estimated_x) = get_liquidity_by_y_sqrt_price(
            y,
            lower_sqrt_price,
            upper_sqrt_price,
            current_sqrt_price,
            rounding_up,
        );
        return (estimated_x, y, liquidity);
    } else if current_sqrt_price < lower_sqrt_price {
        // single token x
        let (liquidity, estimated_y) = get_liquidity_by_x_sqrt_price(
            x,
            lower_sqrt_price,
            upper_sqrt_price,
            current_sqrt_price,
            rounding_up,
        );
        return (x, estimated_y, liquidity);
    }
    let (liquidity_by_y, estimated_x) = get_liquidity_by_y_sqrt_price(
        y,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    );
    let (liquidity_by_x, estimated_y) = get_liquidity_by_x_sqrt_price(
        x,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    );
    if liquidity_by_y < liquidity_by_x {
        (estimated_x, estimated_y, liquidity_by_y)
    } else {
        (estimated_x, estimated_y, liquidity_by_x)
    }
}

pub fn get_liquidity_by_x(
    x: TokenAmount,
    mut lower_tick: i32,
    mut upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
    tick_spacing: u16,
) -> (Liquidity, TokenAmount) {
    if lower_tick < -MAX_TICK {
        lower_tick = get_min_tick(tick_spacing);
    }
    if upper_tick > MAX_TICK {
        upper_tick = get_max_tick(tick_spacing);
    }

    let lower_sqrt_price = calculate_sqrt_price(lower_tick).unwrap();
    let upper_sqrt_price = calculate_sqrt_price(upper_tick).unwrap();

    get_liquidity_by_x_sqrt_price(
        x,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    )
}
pub fn get_liquidity_by_x_sqrt_price(
    x: TokenAmount,
    lower_sqrt_price: SqrtPrice,
    upper_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> (Liquidity, TokenAmount) {
    if upper_sqrt_price < current_sqrt_price {
        // err
        return (Liquidity::new(0), TokenAmount::new(0));
    }

    if current_sqrt_price < lower_sqrt_price {
        // Checked_from_decimal_to_value
        let nominator =
            (lower_sqrt_price.big_mul(upper_sqrt_price)).big_div(SqrtPrice::new(PRICE_DENOMINATOR));
        let denominator = upper_sqrt_price - lower_sqrt_price;
        let liquidity = Liquidity::from_integer((x.0 * nominator.get()) / denominator.get());
        return (liquidity, TokenAmount(0));
    }

    let nominator = current_sqrt_price
        .big_mul(upper_sqrt_price)
        .big_div(SqrtPrice::new(PRICE_DENOMINATOR));
    let denominator = upper_sqrt_price - current_sqrt_price;
    let liquidity = Liquidity::from_integer((x.0 * nominator.get()) / denominator.get());
    let sqrt_price_diff = current_sqrt_price - lower_sqrt_price;
    let y = calculate_y(sqrt_price_diff, liquidity, rounding_up);
    (liquidity, y)
}

pub fn get_liquidity_by_y(
    y: TokenAmount,
    mut lower_tick: i32,
    mut upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
    tick_spacing: u16,
) -> (Liquidity, TokenAmount) {
    if lower_tick < -MAX_TICK {
        lower_tick = get_min_tick(tick_spacing);
    }
    if upper_tick > MAX_TICK {
        upper_tick = get_max_tick(tick_spacing);
    }

    let lower_sqrt_price = calculate_sqrt_price(lower_tick).unwrap();
    let upper_sqrt_price = calculate_sqrt_price(upper_tick).unwrap();

    get_liquidity_by_y_sqrt_price(
        y,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    )
}

pub fn get_liquidity_by_y_sqrt_price(
    y: TokenAmount,
    lower_sqrt_price: SqrtPrice,
    upper_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> (Liquidity, TokenAmount) {
    if current_sqrt_price < lower_sqrt_price {
        // err
        return (Liquidity::new(0), TokenAmount::new(0));
    }

    if upper_sqrt_price <= current_sqrt_price {
        let sqrt_price_diff = upper_sqrt_price - lower_sqrt_price;
        let liquidity = Liquidity::from_integer(y.0 * PRICE_DENOMINATOR / sqrt_price_diff.get());
        return (liquidity, TokenAmount::new(0));
    }

    let sqrt_price_diff = current_sqrt_price - lower_sqrt_price;
    let liquidity = Liquidity::from_integer(y.0 * PRICE_DENOMINATOR / sqrt_price_diff.get());
    let denominator =
        (current_sqrt_price.big_mul(upper_sqrt_price)).big_div(SqrtPrice::new(PRICE_DENOMINATOR));
    let nominator = upper_sqrt_price - current_sqrt_price;

    let x = calculate_x(nominator, denominator, liquidity, rounding_up);

    (liquidity, x)
}

pub fn calculate_x(
    nominator: SqrtPrice,
    denominator: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TokenAmount {
    let common = liquidity.big_mul(nominator).big_div(denominator).get();

    if rounding_up {
        TokenAmount::new(((common + LIQUIDITY_DENOMINATOR) - 1) / LIQUIDITY_DENOMINATOR)
    } else {
        TokenAmount::new(common / LIQUIDITY_DENOMINATOR)
    }
}

pub fn calculate_y(
    sqrt_price_diff: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TokenAmount {
    let shifted_liquidity = liquidity.get() / LIQUIDITY_DENOMINATOR;
    if rounding_up {
        TokenAmount::new(
            ((sqrt_price_diff.get() * shifted_liquidity) + (PRICE_DENOMINATOR - 1))
                / PRICE_DENOMINATOR,
        )
    } else {
        TokenAmount::new(sqrt_price_diff.get() * shifted_liquidity / PRICE_DENOMINATOR)
    }
}

pub fn get_min_tick(tick_spacing: u16) -> i32 {
    let limited_by_price = -MAX_TICK + (-MAX_TICK % tick_spacing as i32);
    let limited_by_tickmap = -TICK_LIMIT * tick_spacing as i32 - tick_spacing as i32;
    if limited_by_price > limited_by_tickmap {
        limited_by_price
    } else {
        limited_by_tickmap
    }
}

pub fn get_max_tick(tick_spacing: u16) -> i32 {
    let limited_by_price = MAX_TICK + (MAX_TICK % tick_spacing as i32);
    let limited_by_tickmap = TICK_LIMIT * tick_spacing as i32 - tick_spacing as i32;
    if limited_by_price > limited_by_tickmap {
        limited_by_price
    } else {
        limited_by_tickmap
    }
}
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
            let result = get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true, 1);
            let expected_l = Liquidity::new(0);
            let expected_y = TokenAmount(0);
            assert_eq!(result.0, expected_l);
            assert_eq!(result.1, expected_y);
        }
        // in current tick
        {
            let expected_l = Liquidity::new(432392997000000);
            let expected_y_up = TokenAmount(434322);
            let expected_y_down = TokenAmount(434321);

            let lower_tick = 80;
            let upper_tick = 120;

            let result_up =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true, 1);

            let result_down =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, false, 1);
            assert_eq!(expected_l, result_up.0);
            assert_eq!(expected_y_up, result_up.1);
            assert_eq!(expected_l, result_down.0);
            assert_eq!(expected_y_down, result_down.1);
        }
        // above current tick
        {
            let expected_l = Liquidity::new(13548826000000);
            let expected_y = TokenAmount(0);
            let lower_tick = 150;
            let upper_tick = 800;

            let result_up =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true, 1);
            assert_eq!(expected_l, result_up.0);
            assert_eq!(expected_y, result_up.1);

            let result_down =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, false, 1);
            assert_eq!(expected_l, result_down.0);
            assert_eq!(expected_y, result_up.1);
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
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true, 1);
            assert_eq!(expected_l, result_up.0);
            assert_eq!(expected_x, result_up.1);
            let result_down =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false, 1);
            assert_eq!(expected_l, result_down.0);
            assert_eq!(expected_x, result_down.1);
        }
        // in current tick
        {
            let expected_l = Liquidity::new(584945290554000000); // PROTOCOL = 584945290554346935
            let expected_x_up = TokenAmount(77539808126);
            let expected_x_down = TokenAmount(77539808125);
            let lower_tick = -25000;
            let upper_tick = -19000;

            let result_up =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true, 1);
            assert_eq!(expected_l, result_up.0);
            assert_eq!(expected_x_up, result_up.1);
            let result_down =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false, 1);
            assert_eq!(expected_l, result_down.0);
            assert_eq!(expected_x_down, result_down.1);
        }
        // above current tick
        {
            let expected_l = Liquidity::new(0);
            let expected_x = TokenAmount(0);
            let lower_tick = -10000;
            let upper_tick = 0;

            let result_up =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true, 1);
            assert_eq!(expected_l, result_up.0);
            assert_eq!(expected_x, result_up.1);
            let result_down =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false, 1);
            assert_eq!(expected_l, result_down.0);
            assert_eq!(expected_x, result_down.1);
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
            let (x_up, _, liquidity_up) = get_liquidity(
                expected_x,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            );
            let (x_down, _, liquidity_down) = get_liquidity(
                expected_x,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            );
            assert_eq!(expected_l, liquidity_up);
            assert_eq!(expected_l, liquidity_down);
            assert_eq!(x_up, expected_x);
            assert_eq!(x_down, expected_x);
        }
        // in current tick
        {
            let lower_tick = -25000;
            let upper_tick = -19000;
            let expected_x_up = TokenAmount(77539808126);
            let expected_x_down = TokenAmount(77539808126); // 77539808125
            let expected_l_up = Liquidity::new(584945290554000000); // 584945290554346935
            let expected_l_down = Liquidity::new(584945290554000000);
            let (x_up, _, liquidity_up) = get_liquidity(
                expected_x_up,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            );
            let (x_down, _, liquidity_down) = get_liquidity(
                expected_x_down,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            );
            assert_eq!(expected_l_up, liquidity_up);
            assert_eq!(expected_l_down, liquidity_down);
            assert_eq!(x_up, expected_x_up);
            assert_eq!(x_down, expected_x_down);
        }
        // above current tick
        {
            let lower_tick = 150;
            let upper_tick = 800;
            let x = TokenAmount(43_000_000_0);
            let expected_y = TokenAmount(0);
            let expected_l = Liquidity::new(13548826311000000); // 13548826311623850
            let (_, y_up, liquidity_up) = get_liquidity(
                x,
                expected_y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            );
            let (_, y_down, liquidity_down) = get_liquidity(
                x,
                expected_y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
                10,
            );
            assert_eq!(expected_l, liquidity_up);
            assert_eq!(expected_l, liquidity_down);
            assert_eq!(y_up, expected_y);
            assert_eq!(y_down, expected_y);
        }
    }
}
