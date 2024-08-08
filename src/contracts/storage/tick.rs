use super::Pool;
use crate::math::types::{
    fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::calculate_sqrt_price,
    sqrt_price::SqrtPrice,
};
use decimal::*;
use traceable_result::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct Tick {
    pub index: i32,
    pub sign: bool,
    pub liquidity_change: Liquidity,
    pub liquidity_gross: Liquidity,
    pub sqrt_price: SqrtPrice,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    pub seconds_outside: u64,
}

pub const MAX_RESULT_SIZE: usize = 16 * 1024 * 8;

// 131072 / (32 + 128 + 8) > 780
pub const LIQUIDITY_TICK_LIMIT: usize = MAX_RESULT_SIZE / (32 + 128 + 8);

#[derive(Debug, Copy, Clone, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct LiquidityTick {
    pub index: i32,
    pub liquidity_change: Liquidity,
    pub sign: bool,
}

impl From<Tick> for LiquidityTick {
    fn from(tick: Tick) -> Self {
        Self {
            index: tick.index,
            liquidity_change: tick.liquidity_change,
            sign: tick.sign,
        }
    }
}

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i32,
            sign: false,
            liquidity_change: Liquidity::new(0),
            liquidity_gross: Liquidity::new(0),
            sqrt_price: SqrtPrice::from_integer(1),
            fee_growth_outside_x: FeeGrowth::new(U256::from(0)),
            fee_growth_outside_y: FeeGrowth::new(U256::from(0)),
            seconds_outside: 0u64,
        }
    }
}

impl Tick {
    pub fn create(index: i32, pool: &Pool, current_timestamp: u64) -> Self {
        let below_current_tick = index <= pool.current_tick_index;

        Self {
            index,
            sign: true,
            sqrt_price: calculate_sqrt_price(index).unwrap(),
            fee_growth_outside_x: match below_current_tick {
                true => pool.fee_growth_global_x,
                false => FeeGrowth::new(U256::from(0)),
            },
            fee_growth_outside_y: match below_current_tick {
                true => pool.fee_growth_global_y,
                false => FeeGrowth::new(U256::from(0)),
            },
            seconds_outside: match below_current_tick {
                true => current_timestamp.checked_sub(pool.start_timestamp).unwrap(),
                false => 0,
            },
            ..Self::default()
        }
    }

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

    pub fn update(
        &mut self,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
        is_upper: bool,
        is_deposit: bool,
    ) -> TrackableResult<()> {
        self.liquidity_gross = self.calculate_new_liquidity_gross(
            is_deposit,
            liquidity_delta,
            max_liquidity_per_tick,
        )?;

        self.update_liquidity_change(liquidity_delta, is_deposit ^ is_upper)?;
        Ok(())
    }

    fn update_liquidity_change(
        &mut self,
        liquidity_delta: Liquidity,
        add: bool,
    ) -> TrackableResult<()> {
        if self.sign ^ add {
            if { self.liquidity_change } > liquidity_delta {
                self.liquidity_change = self
                    .liquidity_change
                    .checked_sub(liquidity_delta)
                    .map_err(|_| err!("Underflow while calculating liquidity change"))?;
            } else {
                self.liquidity_change = liquidity_delta
                    .checked_sub(self.liquidity_change)
                    .map_err(|_| err!("Underflow while calculating liquidity change"))?;
                self.sign = !self.sign;
            }
        } else {
            self.liquidity_change = self
                .liquidity_change
                .checked_add(liquidity_delta)
                .map_err(|_| err!("Overflow while calculating liquidity change"))?;
        }

        Ok(())
    }

    fn calculate_new_liquidity_gross(
        self,
        sign: bool,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
    ) -> TrackableResult<Liquidity> {
        // validate in decrease liquidity case
        if !sign && { self.liquidity_gross } < liquidity_delta {
            return Err(err!("InvalidTickLiquidity"));
        }
        let new_liquidity = match sign {
            true => self
                .liquidity_gross
                .checked_add(liquidity_delta)
                .map_err(|_| err!("tick add liquidity overflow")),
            false => self
                .liquidity_gross
                .checked_sub(liquidity_delta)
                .map_err(|_| err!("tick sun liquidity overflow")),
        }?;
        // validate in increase liquidity case
        if sign && new_liquidity >= max_liquidity_per_tick {
            return Err(err!("InvalidTickLiquidity"));
        }

        Ok(new_liquidity)
    }
}

#[cfg(test)]
mod tests {
    use decimal::{Decimal, Factories};

    use crate::math::clamm::calculate_max_liquidity_per_tick;

    use super::*;

    #[test]
    fn test_cross() {
        {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(45_u128.into()),
                fee_growth_global_y: FeeGrowth::new(35_u128.into()),
                liquidity: Liquidity::from_integer(4),
                last_timestamp: 15,
                start_timestamp: 4,
                current_tick_index: 7,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(30_u128.into()),
                fee_growth_outside_y: FeeGrowth::new(25_u128.into()),
                index: 3,
                seconds_outside: 5,
                liquidity_change: Liquidity::from_integer(1),
                ..Default::default()
            };
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(45_u128.into()),
                fee_growth_global_y: FeeGrowth::new(35_u128.into()),
                liquidity: Liquidity::from_integer(5),
                last_timestamp: 315360015,
                start_timestamp: 4,
                current_tick_index: 7,
                ..Default::default()
            };
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(15_u128.into()),
                fee_growth_outside_y: FeeGrowth::new(10_u128.into()),
                index: 3,
                seconds_outside: 315360006,
                liquidity_change: Liquidity::from_integer(1),
                ..Default::default()
            };
            tick.cross(&mut pool, 315360015).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
        {
            // let mut pool = Pool {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(68_u128.into()),
                fee_growth_global_y: FeeGrowth::new(59_u128.into()),
                liquidity: Liquidity::new(0),
                last_timestamp: 9,
                start_timestamp: 34,
                current_tick_index: 4,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(42_u128.into()),
                fee_growth_outside_y: FeeGrowth::new(14_u128.into()),
                index: 9,
                seconds_outside: 41,
                liquidity_change: Liquidity::new(0),
                ..Default::default()
            };
            // let result_pool = Pool {
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(68_u128.into()),
                fee_growth_global_y: FeeGrowth::new(59_u128.into()),
                liquidity: Liquidity::new(0),
                last_timestamp: 315360000,
                start_timestamp: 34,
                current_tick_index: 4,
                ..Default::default()
            };
            // let result_tick = Tick {
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(26_u128.into()),
                fee_growth_outside_y: FeeGrowth::new(45_u128.into()),
                index: 9,
                seconds_outside: 315359925,
                liquidity_change: Liquidity::from_integer(0),
                ..Default::default()
            };

            tick.cross(&mut pool, 315360000).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
        // fee_growth_outside should underflow
        {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(3402_u128.into()),
                fee_growth_global_y: FeeGrowth::new(3401_u128.into()),
                liquidity: Liquidity::from_integer(14),
                last_timestamp: 9,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(26584_u128.into()),
                fee_growth_outside_y: FeeGrowth::new(1256588_u128.into()),
                index: 45,
                seconds_outside: 74,
                liquidity_change: Liquidity::new(10),
                ..Default::default()
            };
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(3402_u128.into()),
                fee_growth_global_y: FeeGrowth::new(3401_u128.into()),
                liquidity: Liquidity::new(13999990),
                last_timestamp: 31536000,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129616754").unwrap()),
                fee_growth_outside_y: FeeGrowth::new(U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913128386749").unwrap()),
                index: 45,
                seconds_outside: 31535911,
                liquidity_change: Liquidity::new(10),
                ..Default::default()
            };

            tick.cross(&mut pool, 31536000).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
        // seconds_per_liquidity_outside should underflow
        {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(145_u128.into()),
                fee_growth_global_y: FeeGrowth::new(364_u128.into()),
                liquidity: Liquidity::new(14),
                last_timestamp: 16,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(99_u128.into()),
                fee_growth_outside_y: FeeGrowth::new(256_u128.into()),
                index: 45,
                seconds_outside: 74,
                liquidity_change: Liquidity::new(10),
                ..Default::default()
            };
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(145_u128.into()),
                fee_growth_global_y: FeeGrowth::new(364_u128.into()),
                liquidity: Liquidity::new(4),
                last_timestamp: 315360000,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(46_u128.into()),
                fee_growth_outside_y: FeeGrowth::new(108_u128.into()),
                index: 45,
                seconds_outside: 315359911,
                liquidity_change: Liquidity::new(10),
                ..Default::default()
            };

            tick.cross(&mut pool, 315360000).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
    }

    #[test]
    fn test_update_liquidity_change() {
        // update when tick sign and sign of liquidity change are the same
        {
            let mut tick = Tick {
                sign: true,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = true;
            let _ = tick.update_liquidity_change(liquidity_delta, add);

            assert!(tick.sign);
            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(5));
        }
        {
            let mut tick = Tick {
                sign: false,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = false;
            let _ = tick.update_liquidity_change(liquidity_delta, add);

            assert!(!tick.sign);
            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(5));
        }
        // update when tick sign and sign of liquidity change are different
        {
            let mut tick = Tick {
                sign: true,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = false;
            let _ = tick.update_liquidity_change(liquidity_delta, add);

            assert!(!tick.sign);
            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(1));
        }
        {
            let mut tick = Tick {
                sign: false,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = true;
            let _ = tick.update_liquidity_change(liquidity_delta, add);

            assert!(tick.sign);
            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(1));
        }
    }

    #[test]
    fn test_update() {
        let max_liquidity = Liquidity::new(u128::MAX);
        {
            let mut tick = Tick {
                index: 0,
                sign: true,
                liquidity_change: Liquidity::from_integer(2),
                liquidity_gross: Liquidity::from_integer(2),
                fee_growth_outside_x: FeeGrowth::from_integer(2),
                fee_growth_outside_y: FeeGrowth::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta: Liquidity = Liquidity::from_integer(1);
            let is_upper: bool = false;
            let is_deposit: bool = true;

            tick.update(liquidity_delta, max_liquidity, is_upper, is_deposit)
                .unwrap();

            assert!(tick.sign);
            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(3));
            assert_eq!({ tick.liquidity_gross }, Liquidity::from_integer(3));
            assert_eq!({ tick.fee_growth_outside_x }, FeeGrowth::from_integer(2));
            assert_eq!({ tick.fee_growth_outside_y }, FeeGrowth::from_integer(2));
        }
        {
            let mut tick = Tick {
                index: 5,
                sign: true,
                liquidity_change: Liquidity::from_integer(3),
                liquidity_gross: Liquidity::from_integer(7),
                fee_growth_outside_x: FeeGrowth::from_integer(13),
                fee_growth_outside_y: FeeGrowth::from_integer(11),
                ..Default::default()
            };
            let liquidity_delta: Liquidity = Liquidity::from_integer(1);
            let is_upper: bool = true;
            let is_deposit: bool = true;

            tick.update(liquidity_delta, max_liquidity, is_upper, is_deposit)
                .unwrap();

            assert!(tick.sign);
            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(2));
            assert_eq!({ tick.liquidity_gross }, Liquidity::from_integer(8));
            assert_eq!({ tick.fee_growth_outside_x }, FeeGrowth::from_integer(13));
            assert_eq!({ tick.fee_growth_outside_y }, FeeGrowth::from_integer(11));
        }
        // exceed max tick liquidity
        {
            let mut tick = Tick {
                index: 5,
                sign: true,
                liquidity_change: Liquidity::from_integer(100_000),
                liquidity_gross: Liquidity::from_integer(100_000),
                fee_growth_outside_x: FeeGrowth::from_integer(1000),
                fee_growth_outside_y: FeeGrowth::from_integer(1000),
                ..Default::default()
            };

            let max_liquidity_per_tick = calculate_max_liquidity_per_tick(1);
            let liquidity_delta = max_liquidity_per_tick + Liquidity::new(1);
            let result = tick.update(liquidity_delta, max_liquidity_per_tick, false, true);
            assert!(result.is_err());
        }
    }
}
