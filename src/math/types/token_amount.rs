use decimal::*;
use traceable_result::*;

use super::sqrt_price::SqrtPrice;

#[decimal(0)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]

pub struct TokenAmount(pub u128);

impl TokenAmount {
    pub fn from_big_sqrt_price(value: U256) -> TrackableResult<TokenAmount> {
        let result: u128 = value
            .checked_div(U256::from(SqrtPrice::one().get()))
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;

        Ok(TokenAmount(result))
    }

    pub fn from_big_sqrt_price_up(value: U256) -> TrackableResult<TokenAmount> {
        let result: u128 = value
            .checked_add(U256::from(SqrtPrice::almost_one().get()))
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(U256::from(SqrtPrice::one().get()))
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;
        Ok(TokenAmount(result))
    }
}
