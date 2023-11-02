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
        let nominator = (lower_sqrt_price * upper_sqrt_price)
            .checked_div(SqrtPrice::new(PRICE_DENOMINATOR))
            .unwrap();
        let denominator = upper_sqrt_price - lower_sqrt_price;
        let liquidity =
            Liquidity::new((x.0 * nominator.get() * LIQUIDITY_DENOMINATOR) / denominator.get());
        return (liquidity, TokenAmount(0));
    }

    let nominator = lower_sqrt_price
        .big_mul(upper_sqrt_price)
        .big_div(SqrtPrice::new(PRICE_DENOMINATOR));
    let denominator = upper_sqrt_price - current_sqrt_price;
    let liquidity =
        Liquidity::new((x.0 * nominator.get() * LIQUIDITY_DENOMINATOR) / denominator.get());
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
        let liquidity =
            Liquidity::new(y.0 * LIQUIDITY_DENOMINATOR * PRICE_DENOMINATOR / sqrt_price_diff.get());
        return (liquidity, TokenAmount::new(0));
    }

    let sqrt_price_diff = current_sqrt_price - lower_sqrt_price;
    let liquidity =
        Liquidity::new(y.0 * LIQUIDITY_DENOMINATOR * PRICE_DENOMINATOR / sqrt_price_diff.get());
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
    let common = liquidity.get() * nominator.get() / denominator.get();

    if rounding_up {
        TokenAmount::new(common + LIQUIDITY_DENOMINATOR - 1 / LIQUIDITY_DENOMINATOR)
    } else {
        TokenAmount::new(common / LIQUIDITY_DENOMINATOR)
    }
}

pub fn calculate_y(
    sqrt_price_diff: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TokenAmount {
    let shifted_liquidity = liquidity / Liquidity::from_integer(1);
    if rounding_up {
        // there should be a decimal functiion to mul_up
        TokenAmount::new(
            sqrt_price_diff.get() * shifted_liquidity.get()
                + (PRICE_DENOMINATOR - 1) / PRICE_DENOMINATOR,
        )
    } else {
        TokenAmount::new(sqrt_price_diff.get() * shifted_liquidity.get() / PRICE_DENOMINATOR)
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
        let expected_liquidity: Liquidity;
        let expected_y: TokenAmount;
        let x = TokenAmount(1);
        let lower_tick = -10;
        let upper_tick = 10;
        let current_sqrt_price = SqrtPrice::from_integer(1);
        let result = get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true, 10);
        println!("Result = {:?}", result);
        // Result = (Liquidity { v: 199960000000 }, TokenAmount(99950012998600000000000000000000))
    }

    #[test]
    fn get_liquidity_by_y_test() {
        let expected_liquidity: Liquidity;
        let expected_x: TokenAmount;
        let y = TokenAmount(1);
        let lower_tick = -10;
        let upper_tick = 10;
        let current_sqrt_price = SqrtPrice::from_integer(1);
        let result = get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true, 10);
        println!("Result = {:?}", result);
    }
    #[test]
    fn get_liquidity_test() {
        let expected_liquidity: Liquidity;
        let expected_x: TokenAmount;
        let expected_y: TokenAmount;
        let x = TokenAmount(1);
        let y = TokenAmount(1);
        let lower_tick = -10;
        let upper_tick = 10;
        let current_sqrt_price = SqrtPrice::from_integer(1);
        let result = get_liquidity(x, y, lower_tick, upper_tick, current_sqrt_price, true, 10);
        println!("Result = {:?}", result);
    }
}
