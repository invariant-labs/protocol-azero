use crate::liquidity::*;

use crate::token_amount::TokenAmount;
use crate::{convert, decimal_ops_uint};
use core::convert::{TryFrom, TryInto};
use decimal::num_traits::{WrappingAdd, WrappingSub};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(28, U512)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FeeGrowth(#[tsify(type = "bigint")] pub U256);

decimal_ops_uint!(FeeGrowth);

impl FeeGrowth {
    pub fn unchecked_add(self, other: FeeGrowth) -> FeeGrowth {
        FeeGrowth::new(self.get().wrapping_add(&other.get()))
    }

    pub fn unchecked_sub(self, other: FeeGrowth) -> FeeGrowth {
        FeeGrowth::new(self.get().wrapping_sub(&other.get()))
    }

    pub fn from_fee(liquidity: Liquidity, fee: TokenAmount) -> TrackableResult<Self> {
        Ok(Self::new(
            U256::from(fee.get())
                .checked_mul(FeeGrowth::one().get())
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_mul(U256::from(Liquidity::one().get()))
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_div(liquidity.here())
                .ok_or_else(|| err!(TrackableError::DIV))?,
        ))
    }

    pub fn to_fee(self, liquidity: Liquidity) -> TrackableResult<TokenAmount> {
        Ok(TokenAmount::new(
            self.cast::<U384>()
                .checked_mul(liquidity.cast::<U384>())
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_div(U384::uint_cast(U256::from(10).pow(U256::from(
                    FeeGrowth::scale().checked_add(Liquidity::scale()).unwrap(),
                ))))
                .ok_or_else(|| err!(TrackableError::DIV))?
                .try_into()
                .map_err(|_| err!(TrackableError::cast::<TokenAmount>().as_str()))?,
        ))
    }
}

#[allow(clippy::too_many_arguments)]
pub fn calculate_fee_growth_inside(
    tick_lower: i32,
    tick_lower_fee_growth_outside_x: FeeGrowth,
    tick_lower_fee_growth_outside_y: FeeGrowth,
    tick_upper: i32,
    tick_upper_fee_growth_outside_x: FeeGrowth,
    tick_upper_fee_growth_outside_y: FeeGrowth,
    tick_current: i32,
    fee_growth_global_x: FeeGrowth,
    fee_growth_global_y: FeeGrowth,
) -> (FeeGrowth, FeeGrowth) {
    // determine position relative to current tick
    let current_above_lower = tick_current >= tick_lower;
    let current_below_upper = tick_current < tick_upper;

    // calculate fee growth below
    let fee_growth_below_x = if current_above_lower {
        tick_lower_fee_growth_outside_x
    } else {
        fee_growth_global_x.unchecked_sub(tick_lower_fee_growth_outside_x)
    };
    let fee_growth_below_y = if current_above_lower {
        tick_lower_fee_growth_outside_y
    } else {
        fee_growth_global_y.unchecked_sub(tick_lower_fee_growth_outside_y)
    };

    // calculate fee growth above
    let fee_growth_above_x = if current_below_upper {
        tick_upper_fee_growth_outside_x
    } else {
        fee_growth_global_x.unchecked_sub(tick_upper_fee_growth_outside_x)
    };
    let fee_growth_above_y = if current_below_upper {
        tick_upper_fee_growth_outside_y
    } else {
        fee_growth_global_y.unchecked_sub(tick_upper_fee_growth_outside_y)
    };

    // calculate fee growth inside
    let fee_growth_inside_x = fee_growth_global_x
        .unchecked_sub(fee_growth_below_x)
        .unchecked_sub(fee_growth_above_x);
    let fee_growth_inside_y = fee_growth_global_y
        .unchecked_sub(fee_growth_below_y)
        .unchecked_sub(fee_growth_above_y);

    (fee_growth_inside_x, fee_growth_inside_y)
}
