use crate::{
    contract::{CalculateSwapResult, Hop, QuoteResult},
    contracts::{FeeTier, FeeTierKey, Pool, PoolKey, Position, Tick},
    math::{
        liquidity::Liquidity, percentage::Percentage, sqrt_price::sqrt_price::SqrtPrice,
        token_amount::TokenAmount,
    },
    InvariantError,
};
use alloc::vec::Vec;
use ink::primitives::AccountId;

#[ink::trait_definition]
pub trait Invariant {
    #[ink(message)]
    fn get_protocol_fee(&self) -> Percentage;

    #[ink(message)]
    fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError>;

    #[ink(message)]
    fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError>;

    #[ink(message)]
    fn change_fee_receiver(
        &mut self,
        pool_key: PoolKey,
        fee_receiver: AccountId,
    ) -> Result<(), InvariantError>;

    #[ink(message)]
    fn create_position(
        &mut self,
        pool_key: PoolKey,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: Liquidity,
        slippage_limit_lower: SqrtPrice,
        slippage_limit_upper: SqrtPrice,
    ) -> Result<Position, InvariantError>;

    #[ink(message)]
    fn swap(
        &mut self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError>;

    #[ink(message)]
    fn swap_route(
        &mut self,
        amount_in: TokenAmount,
        expected_amount_out: TokenAmount,
        slippage: Percentage,
        swaps: Vec<Hop>,
    ) -> Result<(), InvariantError>;

    #[ink(message)]
    fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<QuoteResult, InvariantError>;

    #[ink(message)]
    fn quote_route(
        &mut self,
        amount_in: TokenAmount,
        swaps: Vec<Hop>,
    ) -> Result<TokenAmount, InvariantError>;

    #[ink(message)]
    fn transfer_position(&mut self, index: u32, receiver: AccountId) -> Result<(), InvariantError>;

    #[ink(message)]
    fn get_position(&mut self, index: u32) -> Result<Position, InvariantError>;

    #[ink(message)]
    fn get_all_positions(&mut self) -> Vec<Position>;

    #[ink(message)]
    fn update_position_seconds_per_liquidity(
        &mut self,
        index: u32,
        pool_key: PoolKey,
    ) -> Result<(), InvariantError>;

    #[ink(message)]
    fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError>;

    #[ink(message)]
    fn remove_position(&mut self, index: u32)
        -> Result<(TokenAmount, TokenAmount), InvariantError>;

    #[ink(message)]
    fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;

    #[ink(message)]
    fn get_fee_tier(&self, key: FeeTierKey) -> Option<()>;

    #[ink(message)]
    fn remove_fee_tier(&mut self, key: FeeTierKey) -> Result<(), InvariantError>;

    #[ink(message)]
    fn create_pool(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
        init_tick: i32,
    ) -> Result<(), InvariantError>;

    #[ink(message)]
    fn get_pool(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
    ) -> Result<Pool, InvariantError>;

    #[ink(message)]
    fn get_pools(&self) -> Vec<PoolKey>;

    #[ink(message)]
    fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError>;

    #[ink(message)]
    fn get_tickmap_bit(&self, key: PoolKey, index: i32) -> bool;
}
