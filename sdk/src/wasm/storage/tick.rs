use crate::{
    types::{fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::SqrtPrice},
    Pool,
};
use decimal::*;
use traceable_result::*;

use crate::alloc::string::ToString;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Tsify, Eq, PartialEq)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Tick {
    #[tsify(type = "bigint")]
    pub index: i64,
    pub sign: bool,
    pub liquidity_change: Liquidity,
    pub liquidity_gross: Liquidity,
    pub sqrt_price: SqrtPrice,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    #[tsify(type = "bigint")]
    pub seconds_outside: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PositionTick {
    #[tsify(type = "bigint")]
    pub index: i32,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    #[tsify(type = "bigint")]
    pub seconds_outside: u64,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityTick {
    #[tsify(type = "bigint")]
    pub index: i64,
    pub liquidity_change: Liquidity,
    pub sign: bool,
}

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i64,
            sign: false,
            liquidity_change: Liquidity::new(0),
            liquidity_gross: Liquidity::new(0),
            sqrt_price: SqrtPrice::from_integer(1),
            fee_growth_outside_x: FeeGrowth::new(0),
            fee_growth_outside_y: FeeGrowth::new(0),
            seconds_outside: 0u64,
        }
    }
}

impl Tick {
    pub fn cross(&mut self, pool: &mut Pool, current_timestamp: u64) -> TrackableResult<()> {
        self.fee_growth_outside_x = pool
            .fee_growth_global_x
            .unchecked_sub(self.fee_growth_outside_x);
        self.fee_growth_outside_y = pool
            .fee_growth_global_y
            .unchecked_sub(self.fee_growth_outside_y);

        let seconds_passed: u64 = current_timestamp
            .checked_sub(pool.start_timestamp)
            .ok_or_else(|| err!("current_timestamp - pool.start_timestamp underflow"))?;
        self.seconds_outside = seconds_passed.wrapping_sub(self.seconds_outside);

        pool.last_timestamp = current_timestamp;

        // When going to higher tick net_liquidity should be added and for going lower subtracted
        if (pool.current_tick_index >= self.index) ^ self.sign {
            pool.liquidity = pool
                .liquidity
                .checked_add(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity + tick.liquidity_change overflow"))?;
        } else {
            pool.liquidity = pool
                .liquidity
                .checked_sub(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity - tick.liquidity_change underflow"))?
        }

        Ok(())
    }
}

impl Default for LiquidityTick {
    fn default() -> Self {
        LiquidityTick {
            index: 0,
            liquidity_change: Liquidity::new(0),
            sign: false,
        }
    }
}

impl LiquidityTick {
    pub fn cross(&mut self, pool: &mut Pool, current_timestamp: u64) -> TrackableResult<()> {
        pool.last_timestamp = current_timestamp;

        // When going to higher tick net_liquidity should be added and for going lower subtracted
        if (pool.current_tick_index >= self.index) ^ self.sign {
            pool.liquidity = pool
                .liquidity
                .checked_add(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity + tick.liquidity_change overflow"))?;
        } else {
            pool.liquidity = pool
                .liquidity
                .checked_sub(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity - tick.liquidity_change underflow"))?
        }

        Ok(())
    }
}
