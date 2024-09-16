use super::{tickmap::MAX_RESULT_SIZE, Pool, PoolKey, Tick, POOL_KEY_SIZE};
use crate::{
    contracts::InvariantError,
    math::{
        clamm::*,
        types::{
            fee_growth::{calculate_fee_growth_inside, FeeGrowth},
            liquidity::Liquidity,
            seconds_per_liquidity::SecondsPerLiquidity,
            sqrt_price::SqrtPrice,
            token_amount::TokenAmount,
        },
    },
};
use decimal::*;
use traceable_result::*;

pub const POSITION_SIZE: usize = POOL_KEY_SIZE + 128 + 32 + 32 + 128 + 128 + 64 + 128 + 128;
pub const MAX_POSITIONS_RETURNED: u32 = (MAX_RESULT_SIZE / POSITION_SIZE) as u32;

#[derive(PartialEq, Default, Debug, Copy, Clone)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct Position {
    pub pool_key: PoolKey,
    pub liquidity: Liquidity,
    pub lower_tick_index: i32,
    pub upper_tick_index: i32,
    pub fee_growth_inside_x: FeeGrowth,
    pub fee_growth_inside_y: FeeGrowth,
    pub last_block_number: u64,
    pub tokens_owed_x: TokenAmount,
    pub tokens_owed_y: TokenAmount,
    pub created_at: u64,
    pub seconds_per_liquidity_inside: SecondsPerLiquidity,
}

impl Position {
    #[allow(clippy::too_many_arguments)]
    pub fn modify(
        &mut self,
        pool: &mut Pool,
        upper_tick: &mut Tick,
        lower_tick: &mut Tick,
        liquidity_delta: Liquidity,
        add: bool,
        current_timestamp: u64,
        tick_spacing: u16,
    ) -> TrackableResult<(TokenAmount, TokenAmount)> {
        if !pool.liquidity.is_zero() && current_timestamp > pool.last_timestamp {
            ok_or_mark_trace!(pool.update_seconds_per_liquidity_global(current_timestamp))?;
        } else {
            pool.last_timestamp = current_timestamp;
        }
        // calculate dynamically limit allows easy modification
        let max_liquidity_per_tick = calculate_max_liquidity_per_tick(tick_spacing);

        // update initialized tick
        lower_tick.update(liquidity_delta, max_liquidity_per_tick, false, add)?;

        upper_tick.update(liquidity_delta, max_liquidity_per_tick, true, add)?;

        // update fee inside position
        let (fee_growth_inside_x, fee_growth_inside_y) = calculate_fee_growth_inside(
            lower_tick.index,
            lower_tick.fee_growth_outside_x,
            lower_tick.fee_growth_outside_y,
            upper_tick.index,
            upper_tick.fee_growth_outside_x,
            upper_tick.fee_growth_outside_y,
            pool.current_tick_index,
            pool.fee_growth_global_x,
            pool.fee_growth_global_y,
        );

        self.update(
            add,
            liquidity_delta,
            fee_growth_inside_x,
            fee_growth_inside_y,
        )?;

        // calculate tokens amounts and update pool liquidity
        ok_or_mark_trace!(pool.update_liquidity(
            liquidity_delta,
            add,
            upper_tick.index,
            lower_tick.index
        ))
    }

    pub fn update(
        &mut self,
        sign: bool,
        liquidity_delta: Liquidity,
        fee_growth_inside_x: FeeGrowth,
        fee_growth_inside_y: FeeGrowth,
    ) -> TrackableResult<()> {
        if liquidity_delta.is_zero() && self.liquidity.is_zero() {
            return Err(err!("EmptyPositionPokes"));
        }

        // calculate accumulated fee
        let tokens_owed_x = ok_or_mark_trace!(fee_growth_inside_x
            .unchecked_sub(self.fee_growth_inside_x)
            .to_fee(self.liquidity))?;
        let tokens_owed_y = ok_or_mark_trace!(fee_growth_inside_y
            .unchecked_sub(self.fee_growth_inside_y)
            .to_fee(self.liquidity))?;

        self.liquidity = ok_or_mark_trace!(self.calculate_new_liquidity(sign, liquidity_delta))?;
        self.fee_growth_inside_x = fee_growth_inside_x;
        self.fee_growth_inside_y = fee_growth_inside_y;

        self.tokens_owed_x = self
            .tokens_owed_x
            .checked_add(tokens_owed_x)
            .map_err(|_| err!("Overflow while calculating tokens owed X"))?;
        self.tokens_owed_y = self
            .tokens_owed_y
            .checked_add(tokens_owed_y)
            .map_err(|_| err!("Overflow while calculating tokens owed Y"))?;
        Ok(())
    }

    fn calculate_new_liquidity(
        &mut self,
        sign: bool,
        liquidity_delta: Liquidity,
    ) -> TrackableResult<Liquidity> {
        // validate in decrease liquidity case
        if !sign && { self.liquidity } < liquidity_delta {
            return Err(err!("InsufficientLiquidity"));
        }

        match sign {
            true => self
                .liquidity
                .checked_add(liquidity_delta)
                .map_err(|_| err!("position add liquidity overflow")),
            false => self
                .liquidity
                .checked_sub(liquidity_delta)
                .map_err(|_| err!("position sub liquidity underflow")),
        }
    }

    pub fn claim_fee(
        &mut self,
        pool: &mut Pool,
        upper_tick: &mut Tick,
        lower_tick: &mut Tick,
        current_timestamp: u64,
    ) -> (TokenAmount, TokenAmount) {
        unwrap!(self.modify(
            pool,
            upper_tick,
            lower_tick,
            Liquidity::new(0),
            true,
            current_timestamp,
            self.pool_key.fee_tier.tick_spacing
        ));

        let tokens_owed_x = self.tokens_owed_x;
        let tokens_owed_y = self.tokens_owed_y;

        self.tokens_owed_x = TokenAmount(0);
        self.tokens_owed_y = TokenAmount(0);

        (tokens_owed_x, tokens_owed_y)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        pool: &mut Pool,
        pool_key: PoolKey,
        lower_tick: &mut Tick,
        upper_tick: &mut Tick,
        current_timestamp_in_milliseconds: u64,
        liquidity_delta: Liquidity,
        slippage_limit_lower: SqrtPrice,
        slippage_limit_upper: SqrtPrice,
        block_number: u64,
        tick_spacing: u16,
    ) -> Result<(Self, TokenAmount, TokenAmount), InvariantError> {
        if pool.sqrt_price < slippage_limit_lower || pool.sqrt_price > slippage_limit_upper {
            return Err(InvariantError::PriceLimitReached);
        }

        // init position
        let mut position = Position {
            pool_key,
            liquidity: Liquidity::new(0),
            lower_tick_index: lower_tick.index,
            upper_tick_index: upper_tick.index,
            fee_growth_inside_x: FeeGrowth::new(0),
            fee_growth_inside_y: FeeGrowth::new(0),
            last_block_number: block_number,
            tokens_owed_x: TokenAmount::new(0),
            tokens_owed_y: TokenAmount::new(0),
            created_at: current_timestamp_in_milliseconds,
            seconds_per_liquidity_inside: SecondsPerLiquidity::new(0),
        };

        let current_timestamp_in_seconds = current_timestamp_in_milliseconds / 1000;
        let (required_x, required_y) = unwrap!(position.modify(
            pool,
            upper_tick,
            lower_tick,
            liquidity_delta,
            true,
            current_timestamp_in_seconds,
            tick_spacing
        ));

        Ok((position, required_x, required_y))
    }

    pub fn remove(
        &mut self,
        pool: &mut Pool,
        current_timestamp: u64,
        lower_tick: &mut Tick,
        upper_tick: &mut Tick,
        tick_spacing: u16,
    ) -> (TokenAmount, TokenAmount, bool, bool) {
        let liquidity_delta = self.liquidity;
        let (mut amount_x, mut amount_y) = unwrap!(self.modify(
            pool,
            upper_tick,
            lower_tick,
            liquidity_delta,
            false,
            current_timestamp,
            tick_spacing
        ));

        amount_x = amount_x.checked_add(self.tokens_owed_x).unwrap();
        amount_y = amount_y.checked_add(self.tokens_owed_y).unwrap();

        let deinitialize_lower_tick = lower_tick.liquidity_gross.is_zero();
        let deinitialize_upper_tick = upper_tick.liquidity_gross.is_zero();

        (
            amount_x,
            amount_y,
            deinitialize_lower_tick,
            deinitialize_upper_tick,
        )
    }

    pub fn update_seconds_per_liquidity(
        &mut self,
        pool: &mut Pool,
        lower_tick: Tick,
        upper_tick: Tick,
        current_timestamp: u64,
        current_block_number: u64,
    ) {
        self.seconds_per_liquidity_inside = pool
            .update_seconds_per_liquidity_inside(
                lower_tick.index,
                lower_tick.seconds_per_liquidity_outside,
                upper_tick.index,
                upper_tick.seconds_per_liquidity_outside,
                current_timestamp,
            )
            .unwrap();

        self.last_block_number = current_block_number;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_new_liquidity() {
        // negative liquidity error
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(1),
                ..Default::default()
            };
            let sign: bool = false;
            let liquidity_delta = Liquidity::from_integer(2);

            let result = position.calculate_new_liquidity(sign, liquidity_delta);

            assert!(result.is_err());
        }
        // adding liquidity
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(2),
                ..Default::default()
            };
            let sign: bool = true;
            let liquidity_delta = Liquidity::from_integer(2);

            let new_liquidity = position
                .calculate_new_liquidity(sign, liquidity_delta)
                .unwrap();

            assert_eq!(new_liquidity, Liquidity::from_integer(4));
        }
        // subtracting liquidity
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(2),
                ..Default::default()
            };
            let sign: bool = false;
            let liquidity_delta = Liquidity::from_integer(2);

            let new_liquidity = position
                .calculate_new_liquidity(sign, liquidity_delta)
                .unwrap();

            assert_eq!(new_liquidity, Liquidity::from_integer(0));
        }
    }

    #[test]
    fn test_update() {
        // Disable empty position pokes error
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(0),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(0);
            let fee_growth_inside_x = FeeGrowth::from_integer(1);
            let fee_growth_inside_y = FeeGrowth::from_integer(1);

            let result = position.update(
                sign,
                liquidity_delta,
                fee_growth_inside_x,
                fee_growth_inside_y,
            );

            assert!(result.is_err());
        }
        // zero liquidity fee shouldn't change
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(0),
                fee_growth_inside_x: FeeGrowth::from_integer(4),
                fee_growth_inside_y: FeeGrowth::from_integer(4),
                tokens_owed_x: TokenAmount(100),
                tokens_owed_y: TokenAmount(100),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(1);
            let fee_growth_inside_x = FeeGrowth::from_integer(5);
            let fee_growth_inside_y = FeeGrowth::from_integer(5);

            position
                .update(
                    sign,
                    liquidity_delta,
                    fee_growth_inside_x,
                    fee_growth_inside_y,
                )
                .unwrap();

            assert_eq!({ position.liquidity }, Liquidity::from_integer(1));
            assert_eq!({ position.fee_growth_inside_x }, FeeGrowth::from_integer(5));
            assert_eq!({ position.fee_growth_inside_y }, FeeGrowth::from_integer(5));
            assert_eq!({ position.tokens_owed_x }, TokenAmount(100));
            assert_eq!({ position.tokens_owed_y }, TokenAmount(100));
        }
        // fee should change
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(1),
                fee_growth_inside_x: FeeGrowth::from_integer(4),
                fee_growth_inside_y: FeeGrowth::from_integer(4),
                tokens_owed_x: TokenAmount(100),
                tokens_owed_y: TokenAmount(100),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(1);
            let fee_growth_inside_x = FeeGrowth::from_integer(5);
            let fee_growth_inside_y = FeeGrowth::from_integer(5);

            position
                .update(
                    sign,
                    liquidity_delta,
                    fee_growth_inside_x,
                    fee_growth_inside_y,
                )
                .unwrap();

            assert_eq!({ position.liquidity }, Liquidity::from_integer(2));
            assert_eq!({ position.fee_growth_inside_x }, FeeGrowth::from_integer(5));
            assert_eq!({ position.fee_growth_inside_y }, FeeGrowth::from_integer(5));
            assert_eq!({ position.tokens_owed_x }, TokenAmount(101));
            assert_eq!({ position.tokens_owed_y }, TokenAmount(101));
        }
        // previous fee_growth_inside close to max and current close to 0
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(1),
                fee_growth_inside_x: FeeGrowth::new(u128::MAX) - FeeGrowth::from_integer(10),
                fee_growth_inside_y: FeeGrowth::new(u128::MAX) - FeeGrowth::from_integer(10),
                tokens_owed_x: TokenAmount(100),
                tokens_owed_y: TokenAmount(100),
                ..Default::default()
            };
            let sign = true;
            let liquidity_delta = Liquidity::from_integer(1);
            let fee_growth_inside_x = FeeGrowth::from_integer(10);
            let fee_growth_inside_y = FeeGrowth::from_integer(10);

            position
                .update(
                    sign,
                    liquidity_delta,
                    fee_growth_inside_x,
                    fee_growth_inside_y,
                )
                .unwrap();

            assert_eq!({ position.liquidity }, Liquidity::from_integer(2));
            assert_eq!(
                { position.fee_growth_inside_x },
                FeeGrowth::from_integer(10)
            );
            assert_eq!(
                { position.fee_growth_inside_y },
                FeeGrowth::from_integer(10)
            );
            assert_eq!({ position.tokens_owed_x }, TokenAmount(120));
            assert_eq!({ position.tokens_owed_y }, TokenAmount(120));
        }
    }

    #[test]
    fn test_modify() {
        // owed tokens after overflow
        {
            let mut position = Position {
                liquidity: Liquidity::from_integer(123),
                fee_growth_inside_x: FeeGrowth::new(u128::MAX) - FeeGrowth::from_integer(1234),
                fee_growth_inside_y: FeeGrowth::new(u128::MAX) - FeeGrowth::from_integer(1234),
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0),
                ..Default::default()
            };
            let mut pool = Pool {
                current_tick_index: 0,
                fee_growth_global_x: FeeGrowth::from_integer(20),
                fee_growth_global_y: FeeGrowth::from_integer(20),
                ..Default::default()
            };
            let mut upper_tick = Tick {
                index: -10,
                fee_growth_outside_x: FeeGrowth::from_integer(15),
                fee_growth_outside_y: FeeGrowth::from_integer(15),
                liquidity_gross: Liquidity::from_integer(123),
                ..Default::default()
            };
            let mut lower_tick = Tick {
                index: -20,
                fee_growth_outside_x: FeeGrowth::from_integer(20),
                fee_growth_outside_y: FeeGrowth::from_integer(20),
                liquidity_gross: Liquidity::from_integer(123),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::new(0);
            let add = true;
            let current_timestamp: u64 = 1234567890;

            position
                .modify(
                    &mut pool,
                    &mut upper_tick,
                    &mut lower_tick,
                    liquidity_delta,
                    add,
                    current_timestamp,
                    1,
                )
                .unwrap();

            assert_eq!({ position.tokens_owed_x }, TokenAmount(151167));
        }
    }
    #[test]
    fn test_update_seconds_per_liquidity() {
        {
            let current_timestamp = 100;

            let mut pool = Pool {
                current_tick_index: 0,
                sqrt_price: SqrtPrice::from_tick(0).unwrap(),
                liquidity: Liquidity::new(20000000000000),
                ..Default::default()
            };

            let mut upper_tick = Tick {
                index: 10,
                liquidity_change: Liquidity::new(10),
                ..Default::default()
            };
            let mut lower_tick = Tick {
                index: -10,
                ..Default::default()
            };
            let pool_before = pool.clone();
            let (mut pos, _, _) = Position::create(
                &mut pool,
                PoolKey::default(),
                &mut lower_tick,
                &mut upper_tick,
                current_timestamp * 1000,
                Liquidity::new(100000000),
                pool_before.sqrt_price,
                pool_before.sqrt_price,
                0,
                1,
            )
            .unwrap();

            assert_eq!(pos.seconds_per_liquidity_inside, SecondsPerLiquidity(0));
            pos.update_seconds_per_liquidity(
                &mut pool,
                lower_tick,
                upper_tick,
                current_timestamp,
                0,
            );

            assert_eq!(
                pos.seconds_per_liquidity_inside,
                SecondsPerLiquidity(5000000000000000000)
            );
            // liquidity change on delta_time == 0
            {
                let _ = Position::create(
                    &mut pool,
                    PoolKey::default(),
                    &mut lower_tick,
                    &mut upper_tick,
                    current_timestamp * 1000,
                    Liquidity::new(100000000),
                    pool_before.sqrt_price,
                    pool_before.sqrt_price,
                    0,
                    1,
                )
                .unwrap();

                pos.update_seconds_per_liquidity(
                    &mut pool,
                    lower_tick,
                    upper_tick,
                    current_timestamp,
                    0,
                );

                assert_eq!(
                    pos.seconds_per_liquidity_inside,
                    SecondsPerLiquidity(5000000000000000000)
                );
            }

            // liquidity change after update on delta_time == 0
            {
                let _ = Position::create(
                    &mut pool,
                    PoolKey::default(),
                    &mut lower_tick,
                    &mut upper_tick,
                    (current_timestamp + 1) * 1000,
                    Liquidity::new(100000000),
                    pool_before.sqrt_price,
                    pool_before.sqrt_price,
                    0,
                    1,
                )
                .unwrap();

                assert_eq!(
                    pos.seconds_per_liquidity_inside,
                    SecondsPerLiquidity(5000000000000000000)
                );

                pos.update_seconds_per_liquidity(
                    &mut pool,
                    lower_tick,
                    upper_tick,
                    current_timestamp + 1,
                    0,
                );

                assert_eq!(
                    pos.seconds_per_liquidity_inside,
                    SecondsPerLiquidity(5049999500004999950)
                );
            }
        }
    }
}
