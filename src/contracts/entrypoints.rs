use crate::{
    contract::{CalculateSwapResult, Hop},
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
    /// Returns the protocol fee represented as a percentage.
    #[ink(message)]
    fn get_protocol_fee(&self) -> Percentage;

    /// Allows an authorized user to withdraw collected fees.
    #[ink(message)]
    fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError>;

    /// Allows an authorized user to adjust the protocol fee.
    #[ink(message)]
    fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError>;

    /// Transfers fee receiver authorization to another user.
    #[ink(message)]
    fn change_fee_receiver(
        &mut self,
        pool_key: PoolKey,
        fee_receiver: AccountId,
    ) -> Result<(), InvariantError>;

    /// This function is used to open a position.
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

    /// Performs a single swap based on the provided parameters.
    #[ink(message)]
    fn swap(
        &mut self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError>;

    /// Performs multiple swaps based on the provided parameters.
    #[ink(message)]
    fn swap_route(
        &mut self,
        amount_in: TokenAmount,
        expected_amount_out: TokenAmount,
        slippage: Percentage,
        swaps: Vec<Hop>,
    ) -> Result<(), InvariantError>;

    /// Calculates a single swap output off-chain.
    #[ink(message)]
    fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<(TokenAmount, TokenAmount, SqrtPrice, Vec<Tick>), InvariantError>;

    /// Calculates multiple swaps output off-chain.
    #[ink(message)]
    fn quote_route(
        &mut self,
        amount_in: TokenAmount,
        swaps: Vec<Hop>,
    ) -> Result<TokenAmount, InvariantError>;

    /// Transfers a position between users.
    #[ink(message)]
    fn transfer_position(&mut self, index: u32, receiver: AccountId) -> Result<(), InvariantError>;

    /// Returns a single position.
    #[ink(message)]
    fn get_position(&mut self, index: u32) -> Result<Position, InvariantError>;

    /// Returns a vector with all positions that the user has.
    #[ink(message)]
    fn get_all_positions(&mut self) -> Vec<Position>;

    #[ink(message)]
    fn update_position_seconds_per_liquidity(
        &mut self,
        index: u32,
        pool_key: PoolKey,
    ) -> Result<(), InvariantError>;

    /// Allows an authorized user to claim collected fees.
    #[ink(message)]
    fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError>;

    /// Removes a position.
    #[ink(message)]
    fn remove_position(&mut self, index: u32)
        -> Result<(TokenAmount, TokenAmount), InvariantError>;

    /// Allows a user to add a custom fee tier.
    #[ink(message)]
    fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;

    /// Returns a fee tier.
    #[ink(message)]
    fn get_fee_tier(&self, key: FeeTierKey) -> Option<()>;

    /// Removes an existing fee tier.
    #[ink(message)]
    fn remove_fee_tier(&mut self, key: FeeTierKey);

    /// Allows a user to create a custom pool on a specified token pair and fee tier.
    #[ink(message)]
    fn create_pool(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
        init_tick: i32,
    ) -> Result<(), InvariantError>;

    /// Returns a pool that is created on a specified token pair with an associated fee tier.
    #[ink(message)]
    fn get_pool(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
    ) -> Result<Pool, InvariantError>;

    /// Returns a tick at a specified index.
    #[ink(message)]
    fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError>;

    /// Returns a boolean that represents if the tick at a specified index is initialized.
    #[ink(message)]
    fn get_tickmap_bit(&self, key: PoolKey, index: i32) -> bool;
}
