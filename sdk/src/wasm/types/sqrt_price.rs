use crate::consts::*;
use crate::types::{fixed_point::FixedPoint, token_amount::TokenAmount};
use crate::{convert, decimal_ops};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

#[decimal(24)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SqrtPrice(#[tsify(type = "bigint")] pub u128);

decimal_ops!(SqrtPrice);

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

        let intermediate_u320 = nominator
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = U256::uint_checked_cast(intermediate_u320)
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

        let intermediate_u320 = nominator
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_mul(Self::one().cast::<U448>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(denominator.checked_sub(U448::from(1u32)).unwrap())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = U256::uint_checked_cast(intermediate_u320)
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

#[wasm_wrapper]
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

#[wasm_wrapper]
pub fn get_max_tick(tick_spacing: u16) -> TrackableResult<i32> {
    let tick_spacing = tick_spacing as i32;
    MAX_TICK
        .checked_div(tick_spacing)
        .ok_or(err!(TrackableError::DIV))?
        .checked_mul(tick_spacing)
        .ok_or(err!(TrackableError::MUL))
}
#[wasm_wrapper]
pub fn get_min_tick(tick_spacing: u16) -> TrackableResult<i32> {
    let tick_spacing = tick_spacing as i32;
    MIN_TICK
        .checked_div(tick_spacing)
        .ok_or(err!(TrackableError::DIV))?
        .checked_mul(tick_spacing)
        .ok_or(err!(TrackableError::MUL))
}

#[wasm_wrapper]
pub fn get_max_sqrt_price(tick_spacing: u16) -> TrackableResult<SqrtPrice> {
    let max_tick = get_max_tick(tick_spacing)?;
    SqrtPrice::from_tick(max_tick)
}

#[wasm_wrapper]
pub fn get_min_sqrt_price(tick_spacing: u16) -> TrackableResult<SqrtPrice> {
    let min_tick = get_min_tick(tick_spacing)?;
    SqrtPrice::from_tick(min_tick)
}
