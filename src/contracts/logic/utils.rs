use crate::math::{percentage::Percentage, MAX_TICK};
use decimal::{Decimal, Factories};

pub fn fee_to_tick_spacing(fee: Percentage) -> u16 {
    if fee < Percentage::from_scale(1, 2) {
        return 1;
    };

    (fee.get() / Percentage::from_scale(1, 2).get()) as u16
}

pub fn get_max_tick(tick_spacing: u16) -> i32 {
    MAX_TICK - (MAX_TICK % tick_spacing as i32)
}
