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
    /// Retrieves the protocol fee represented as a percentage.
    #[ink(message)]
    fn get_protocol_fee(&self) -> Percentage;

    /// Allows an authorized user to withdraw collected fees.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    ///
    /// # Errors
    /// - Reverts the call when the caller is an unauthorized receiver.
    #[ink(message)]
    fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError>;

    /// Allows an authorized user to adjust the protocol fee.
    ///
    /// # Parameters
    /// - `protocol_fee`: The expected fee represented as a percentage.
    ///
    /// # Errors
    /// - Reverts the call when the caller is an unauthorized user.
    #[ink(message)]
    fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError>;

    /// Transfers fee receiver authorization to another user.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    /// - `fee_receiver`: An `AccountId` identifying the user authorized to claim fees.
    ///
    /// # Errors
    /// - Reverts the call when the caller is an unauthorized user.
    #[ink(message)]
    fn change_fee_receiver(
        &mut self,
        pool_key: PoolKey,
        fee_receiver: AccountId,
    ) -> Result<(), InvariantError>;

    /// Opens a position.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    /// - `lower_tick`: The index of the lower tick for opening the position.
    /// - `upper_tick`: The index of the upper tick for opening the position.
    /// - `liquidity_delta`: The desired liquidity provided by the user in the specified range.
    /// - `slippage_limit_lower`: The price limit for downward movement to execute the position creation.
    /// - `slippage_limit_upper`: The price limit for upward movement to execute the position creation.
    ///
    /// # Events
    /// - On successful transfer, emits a `Create Position` event for the newly opened position.
    ///
    /// # Errors
    /// - Fails if the user attempts to open a position with zero liquidity.
    /// - Fails if the user attempts to create a position with invalid tick indexes or tick spacing.
    /// - Fails if the price has reached the slippage limit.
    /// - Fails if the allowance is insufficient or the user balance transfer fails.
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
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    /// - `x_to_y`: A boolean specifying the swap direction.
    /// - `amount`: TokenAmount that the user wants to swap.
    /// - `by_amount_in`: A boolean specifying whether the user provides the amount to swap or expects the amount out.
    /// - `sqrt_price_limit`: A price limit allowing the price to move for the swap to occur.
    ///
    /// # Events
    /// - On a successful swap, emits a `Swap` event for the freshly made swap.
    /// - On a successful swap, emits a `Cross Tick` event for every single tick crossed.
    ///
    /// # Errors
    /// - Fails if the user attempts to perform a swap with zero amounts.
    /// - Fails if the price has reached the specified limit.
    /// - Fails if the user would receive zero tokens.
    /// - Fails if the allowance is insufficient or the user balance transfer fails.
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
    ///
    /// # Parameters
    /// - `amount_in`: The amount of tokens that the user wants to swap.
    /// - `expected_amount_out`: The amount of tokens that the user wants to receive as a result of the swaps.
    /// - `slippage`: The difference between the expected and actual outcome of a trade represented as a percentage.
    /// - `swaps`: A vector containing all parameters needed to identify separate swap steps.
    ///
    /// # Events
    /// - On every successful swap, emits a `Swap` event for the freshly made swap.
    /// - On every successful swap, emits a `Cross Tick` event for every single tick crossed.
    ///
    /// # Errors
    /// - Fails if the user attempts to perform a swap with zero amounts.
    /// - Fails if the user would receive zero tokens.
    /// - Fails if the allowance is insufficient or the user balance transfer fails.
    /// - Fails if the minimum amount out after a single swap is insufficient to perform the next swap to achieve the expected amount out.
    #[ink(message)]
    fn swap_route(
        &mut self,
        amount_in: TokenAmount,
        expected_amount_out: TokenAmount,
        slippage: Percentage,
        swaps: Vec<Hop>,
    ) -> Result<(), InvariantError>;

    /// Calculates the output of a single off-chain swap.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    /// - `x_to_y`: A boolean specifying the swap direction.
    /// - `amount`: The amount of tokens that the user wants to swap.
    /// - `by_amount_in`: A boolean specifying whether the user provides the amount to swap or expects the amount out.
    /// - `sqrt_price_limit`: A price limit allowing the price to move for the swap to occur.
    ///
    /// # Errors
    /// - Fails if the user attempts to perform a swap with zero amounts.
    /// - Fails if the price has reached the specified limit.
    /// - Fails if the user would receive zero tokens.
    #[ink(message)]
    fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<(TokenAmount, TokenAmount, SqrtPrice, Vec<Tick>), InvariantError>;

    /// Calculates the output of multiple off-chain swaps.
    ///
    /// # Parameters
    /// - `amount_in`: The amount of tokens that the user wants to swap.
    /// - `swaps`: A vector containing all parameters needed to identify separate swap steps.
    ///
    /// # Errors
    /// - Fails if the user attempts to perform a swap with zero amounts.
    /// - Fails if the user would receive zero tokens.
    #[ink(message)]
    fn quote_route(
        &mut self,
        amount_in: TokenAmount,
        swaps: Vec<Hop>,
    ) -> Result<TokenAmount, InvariantError>;

    /// Transfers a position between users.
    ///
    /// # Parameters
    /// - `index`: The index of the user position to transfer.
    /// - `receiver`: An `AccountId` identifying the user who will own the position.
    #[ink(message)]
    fn transfer_position(&mut self, index: u32, receiver: AccountId) -> Result<(), InvariantError>;

    /// Retrieves information about a single position.
    ///
    /// # Parameters
    /// - `index`: The index of the user position.
    #[ink(message)]
    fn get_position(&mut self, index: u32) -> Result<Position, InvariantError>;

    /// Retrieves a vector containing all positions held by the user.
    #[ink(message)]
    fn get_all_positions(&mut self) -> Vec<Position>;

    #[ink(message)]
    fn update_position_seconds_per_liquidity(
        &mut self,
        index: u32,
        pool_key: PoolKey,
    ) -> Result<(), InvariantError>;

    /// Allows an authorized user to claim collected fees.
    ///
    /// # Parameters
    /// - `index`: The index of the user position from which fees will be claimed.
    ///
    /// # Errors
    /// - Fails if the position cannot be found.
    /// - Fails if the DEX has insufficient balance to perform the transfer.
    #[ink(message)]
    fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError>;

    /// Removes a position.
    ///
    /// # Parameters
    /// - `index`: The index of the user position to be removed.
    ///
    /// # Events
    /// - Emits a `Remove Position` event upon success.
    ///
    /// # Errors
    /// - Fails if unable to deinitialize ticks.
    /// - Fails if the DEX has insufficient balance to perform the transfer.
    #[ink(message)]
    fn remove_position(&mut self, index: u32)
        -> Result<(TokenAmount, TokenAmount), InvariantError>;

    /// Allows a user to add a custom fee tier.
    ///
    /// # Parameters
    /// - `fee_tier`: A struct identifying the pool fee and tick spacing.
    ///
    /// # Errors
    /// - Fails if an unauthorized user attempts to create a fee tier.
    /// - Fails if the tick spacing is invalid.
    /// - Fails if the fee tier already exists.
    #[ink(message)]
    fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;

    /// Retrieves information about a fee tier.
    ///
    /// # Parameters
    /// - `fee_tier_key`: A struct identifying the pool fee and tick spacing.
    #[ink(message)]
    fn get_fee_tier(&self, key: FeeTierKey) -> Option<()>;

    /// Removes an existing fee tier.
    ///
    /// # Parameters
    /// - `fee_tier_key`: A struct identifying the pool fee and tick spacing.
    #[ink(message)]
    fn remove_fee_tier(&mut self, key: FeeTierKey);

    /// Allows a user to create a custom pool on a specified token pair and fee tier.
    ///
    /// # Parameters
    /// - `token_0`: The address of the first token.
    /// - `token_1`: The address of the second token.
    /// - `fee_tier`: A struct identifying the pool fee and tick spacing.
    /// - `init_tick`: The initial tick at which the pool will be created.
    ///
    /// # Errors
    /// - Fails if the specified fee tier cannot be found.
    /// - Fails if the user attempts to create a pool for the same tokens.
    #[ink(message)]
    fn create_pool(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
        init_tick: i32,
    ) -> Result<(), InvariantError>;

    /// Retrieves information about a pool created on a specified token pair with an associated fee tier.
    ///
    /// # Parameters
    /// - `token_0`: The address of the first token.
    /// - `token_1`: The address of the second token.
    /// - `fee_tier`: A struct identifying the pool fee and tick spacing.
    ///
    /// # Errors
    /// - Fails if the Pool key cannot be created.
    #[ink(message)]
    fn get_pool(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
    ) -> Result<Pool, InvariantError>;

    /// Retrieves information about a tick at a specified index.
    ///
    /// # Parameters
    /// - `key`: A unique key that identifies the specified pool.
    /// - `index`: The tick index in the tickmap.

    #[ink(message)]
    fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError>;

    /// Checks if the tick at a specified index is initialized.
    ///
    /// # Parameters
    /// - `key`: A unique key that identifies the specified pool.
    /// - `index`: The tick index in the tickmap.
    #[ink(message)]
    fn get_tickmap_bit(&self, key: PoolKey, index: i32) -> bool;
}
