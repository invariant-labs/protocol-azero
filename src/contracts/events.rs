use crate::{
    contracts::PoolKey,
    math::{liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount},
};
use ink::{prelude::vec::Vec, primitives::AccountId};

#[ink::event]
pub struct CreatePositionEvent {
    #[ink(topic)]
    pub timestamp: u64,
    pub address: AccountId,
    pub pool: PoolKey,
    pub liquidity: Liquidity,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}

#[ink::event]
pub struct ChangeLiquidityEvent {
    #[ink(topic)]
    pub timestamp: u64,
    pub address: AccountId,
    pub pool: PoolKey,
    pub old_liquidity: Liquidity,
    pub new_liquidity: Liquidity,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}

#[ink::event]
pub struct CrossTickEvent {
    #[ink(topic)]
    pub timestamp: u64,
    pub address: AccountId,
    pub pool: PoolKey,
    pub indexes: Vec<i32>,
}

#[ink::event]
pub struct RemovePositionEvent {
    #[ink(topic)]
    pub timestamp: u64,
    pub address: AccountId,
    pub pool: PoolKey,
    pub liquidity: Liquidity,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}

#[ink::event]
pub struct SwapEvent {
    #[ink(topic)]
    pub timestamp: u64,
    pub address: AccountId,
    pub pool: PoolKey,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub x_to_y: bool,
}
