use crate::{
    contracts::{Pool, PoolKey, Tick},
    math::types::{sqrt_price::SqrtPrice, token_amount::TokenAmount},
};
use ink::prelude::vec::Vec;

#[derive(Default, Clone, Debug, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CalculateSwapResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub fee: TokenAmount,
    pub pool: Pool,
    pub ticks: Vec<Tick>,
}

#[derive(Default, Debug)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct QuoteResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub target_sqrt_price: SqrtPrice,
    pub ticks: Vec<Tick>,
}

#[derive(Clone, Debug)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct SwapHop {
    pub pool_key: PoolKey,
    pub x_to_y: bool,
}
