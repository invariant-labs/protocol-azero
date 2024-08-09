use core::ops::Mul;

use decimal::*;
use traceable_result::*;

use crate::math::consts::*;
use crate::math::types::{fixed_point::FixedPoint, token_amount::TokenAmount};

#[decimal(24)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
pub struct SqrtPrice(pub u128);

impl SqrtPrice {
    pub fn from_tick(i: i32) -> TrackableResult<Self> {
        calculate_sqrt_price(i)
    }

    pub fn big_div_values_to_token(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<TokenAmount> {
        let nominator = U448::uint_cast(nominator);
        let denominator = U448::uint_cast(denominator);

        let intermediate_u448 = nominator
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = U256::uint_checked_cast(intermediate_u448)
            .map_err(|e| err!(&e))?
            .checked_div(U256::from(Self::one().get()))
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<TokenAmount>().as_str()))?;
        Ok(TokenAmount(result))
    }

    pub fn big_div_values_to_token_up(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<TokenAmount> {
        let nominator = U448::uint_cast(nominator);
        let denominator = U448::uint_cast(denominator);

        let intermediate_u448 = nominator
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(denominator.checked_sub(U448::from(1u32)).unwrap())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = U256::uint_checked_cast(intermediate_u448)
            .map_err(|e| err!(&e))?
            .checked_add(U256::from(Self::almost_one().get()))
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(U256::from(Self::one().get()))
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<TokenAmount>().as_str()))?;
        Ok(TokenAmount::new(result))
    }

    pub fn big_div_values_up(nominator: U256, denominator: U256) -> SqrtPrice {
        SqrtPrice::new({
            nominator
                .checked_mul(U256::from(Self::one().get()))
                .unwrap()
                .checked_add(denominator.checked_sub(U256::from(1u32)).unwrap())
                .unwrap()
                .checked_div(denominator)
                .unwrap()
                .try_into()
                .unwrap()
        })
    }

    pub fn checked_big_div_values(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<SqrtPrice> {
        Ok(SqrtPrice::new(
            nominator
                .checked_mul(U256::from(Self::one().get()))
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_div(denominator)
                .ok_or_else(|| err!(TrackableError::DIV))?
                .try_into()
                .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
    }

    pub fn checked_big_div_values_up(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<SqrtPrice> {
        let denominator = U320::uint_cast(denominator);

        Ok(SqrtPrice::new(
            U320::uint_cast(nominator)
                .checked_mul(Self::one().cast::<U320>())
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_add(
                    denominator
                        .checked_sub(U320::from(1u32))
                        .ok_or_else(|| err!(TrackableError::SUB))?,
                )
                .ok_or_else(|| err!(TrackableError::ADD))?
                .checked_div(denominator)
                .ok_or_else(|| err!(TrackableError::DIV))?
                .try_into()
                .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
    }
}

pub fn check_tick_to_sqrt_price_relationship(
    tick_index: i32,
    tick_spacing: u16,
    sqrt_price: SqrtPrice,
) -> TrackableResult<bool> {
    if tick_index.checked_add(tick_spacing as i32).unwrap() > MAX_TICK {
        let max_tick = get_max_tick(tick_spacing);
        let max_sqrt_price = ok_or_mark_trace!(SqrtPrice::from_tick(max_tick))?;
        if sqrt_price != max_sqrt_price {
            return Ok(false);
        }
    } else {
        let lower_bound = ok_or_mark_trace!(SqrtPrice::from_tick(tick_index))?;
        let upper_bound = ok_or_mark_trace!(SqrtPrice::from_tick(
            tick_index.checked_add(tick_spacing as i32).unwrap()
        ))?;
        if sqrt_price >= upper_bound || sqrt_price < lower_bound {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn calculate_sqrt_price(tick_index: i32) -> TrackableResult<SqrtPrice> {
    // checking if tick be converted to sqrt_price (overflows if more)
    let tick = tick_index.abs();

    if tick > MAX_TICK {
        return Err(err!("tick over bounds"));
    }

    let mut sqrt_price = FixedPoint::one();

    if tick & 0x1 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1000049998750062496094023_u128.into()))
    }
    if tick & 0x2 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1000100000000000000000000_u128.into()))
    }
    if tick & 0x4 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1000200010000000000000000_u128.into()))
    }
    if tick & 0x8 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1000400060004000100000000_u128.into()))
    }
    if tick & 0x10 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1000800280056007000560028_u128.into()))
    }
    if tick & 0x20 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1001601200560182043688009_u128.into()))
    }
    if tick & 0x40 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1003204964963598014666528_u128.into()))
    }
    if tick & 0x80 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1006420201727613920156533_u128.into()))
    }
    if tick & 0x100 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1012881622445451097078095_u128.into()))
    }
    if tick & 0x200 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1025929181087729343658708_u128.into()))
    }
    if tick & 0x400 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1052530684607338948386589_u128.into()))
    }
    if tick & 0x800 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1107820842039993613899215_u128.into()))
    }
    if tick & 0x1000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1227267018058200482050503_u128.into()))
    }
    if tick & 0x2000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(1506184333613467388107955_u128.into()))
    }
    if tick & 0x4000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(2268591246822644826925609_u128.into()))
    }
    if tick & 0x8000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(5146506245160322222537991_u128.into()))
    }
    if tick & 0x0001_0000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(26486526531474198664033811_u128.into()))
    }
    if tick & 0x0002_0000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(701536087702486644953017488_u128.into()))
    }
    if tick & 0x0004_0000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(492152882348911033633683861778_u128.into()))
    }
    if tick & 0x0008_0000 != 0 {
        sqrt_price = sqrt_price.mul(FixedPoint::new(
            242214459604341065650571799093539783_u128.into(),
        ))
    };
    Ok(if tick_index >= 0 {
        SqrtPrice::new(sqrt_price.cast::<u128>())
    } else {
        SqrtPrice::new(u128::uint_cast(
            SqrtPrice::one()
                .cast::<U256>()
                .checked_mul(SqrtPrice::one().here())
                .unwrap()
                .checked_div(sqrt_price.here())
                .ok_or(err!("calculate_sqrt_price::checked_div division failed"))?,
        ))
    })
}

pub fn get_max_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MAX_TICK
        .checked_div(tick_spacing)
        .unwrap()
        .checked_mul(tick_spacing)
        .unwrap()
}

pub fn get_min_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MIN_TICK
        .checked_div(tick_spacing)
        .unwrap()
        .checked_mul(tick_spacing)
        .unwrap()
}

pub fn get_max_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let max_tick = get_max_tick(tick_spacing);
    SqrtPrice::from_tick(max_tick).unwrap()
}

pub fn get_min_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let min_tick = get_min_tick(tick_spacing);
    SqrtPrice::from_tick(min_tick).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sqrt_price() {
        {
            let sqrt_price = SqrtPrice::from_tick(0).unwrap();
            assert_eq!(sqrt_price, SqrtPrice::from_integer(1));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(20_000).unwrap();
            // expected 2.718145926825224864037656
            // real     2.718145926825224864037664...
            assert_eq!(sqrt_price, SqrtPrice::new(2718145926825224864037656));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(200_000).unwrap();
            // expected 22015.456048552198645701365772
            // real     22015.456048552198645701456581......
            assert_eq!(sqrt_price, SqrtPrice::new(22015456048552198645701365772));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(-20_000).unwrap();
            // expected 0.367897834377123709894002
            // real     0.367897834377123709894001...
            assert_eq!(sqrt_price, SqrtPrice::new(367897834377123709894002));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(-200_000).unwrap();
            // expected 0.000045422633889328990341
            // real     0.000045422633889328990341...
            assert_eq!(sqrt_price, SqrtPrice::new(45422633889328990341))
        }
        {
            let sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
            // expected 281481114768267.672330495791029795421271
            // real     281481114768267.672330498244929173903929...
            assert_eq!(
                sqrt_price,
                SqrtPrice::new(281481114768267672330495791029795421271)
            );
            assert_eq!(sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(MIN_TICK).unwrap();
            // expected 0.000000000000003552636207
            // real     0.000000000000003552636207...
            assert_eq!(sqrt_price, SqrtPrice::new(3552636207));
            assert_eq!(sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE));
        }
    }

    #[test]
    fn test_domain_calculate_sqrt_price() {
        // over max tick
        {
            let tick_out_of_range = MAX_TICK + 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
        // below min tick
        {
            let tick_out_of_range = -MAX_TICK - 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
    }

    #[test]
    fn test_sqrt_price_limitation() {
        {
            let global_max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
            assert_eq!(global_max_sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE)); // ceil(log2(this)) = 96
            let global_min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
            assert_eq!(global_min_sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE)); // floor(log2(this)) = 63
        }
        {
            let max_sqrt_price = get_max_sqrt_price(1);
            let max_tick: i32 = get_max_tick(1);
            assert_eq!(max_sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(MAX_SQRT_PRICE)
            );

            let max_sqrt_price = get_max_sqrt_price(2);
            let max_tick: i32 = get_max_tick(2);
            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(281467041767995484175575572190311022481)
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(281467041767995484175575572190311022481)
            );

            let max_sqrt_price = get_max_sqrt_price(5);
            let max_tick: i32 = get_max_tick(5);
            assert_eq!(max_sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(MAX_SQRT_PRICE)
            );

            let max_sqrt_price = get_max_sqrt_price(10);
            let max_tick: i32 = get_max_tick(10);
            assert_eq!(max_tick, 665450);
            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(281410756802527410668167604461672779949)
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(281410756802527410668167604461672779949)
            );

            let max_sqrt_price = get_max_sqrt_price(100);
            let max_tick: i32 = get_max_tick(100);
            assert_eq!(max_tick, 665400);

            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(280708143672930091210419229357187153384)
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(280708143672930091210419229357187153384)
            );
        }
        {
            let min_sqrt_price = get_min_sqrt_price(1);
            let min_tick: i32 = get_min_tick(1);
            assert_eq!(min_sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(MIN_SQRT_PRICE)
            );

            let min_sqrt_price = get_min_sqrt_price(2);
            let min_tick: i32 = get_min_tick(2);
            assert_eq!(min_sqrt_price, SqrtPrice::new(3552813834));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(3552813834)
            );

            let min_sqrt_price = get_min_sqrt_price(5);
            let min_tick: i32 = get_min_tick(5);
            assert_eq!(min_sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(MIN_SQRT_PRICE)
            );

            let min_sqrt_price = get_min_sqrt_price(10);
            let min_tick: i32 = get_min_tick(10);
            assert_eq!(min_tick, -665450);
            assert_eq!(min_sqrt_price, SqrtPrice::new(3553524432));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(3553524432)
            );

            let min_sqrt_price = get_min_sqrt_price(100);
            let min_tick: i32 = get_min_tick(100);
            assert_eq!(min_tick, -665400);
            assert_eq!(min_sqrt_price, SqrtPrice::new(3562418912));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(3562418912)
            );
        }
    }
}
