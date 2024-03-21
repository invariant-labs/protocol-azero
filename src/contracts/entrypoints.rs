use crate::{
    contracts::{FeeTier, LiquidityTick, Pool, PoolKey, Position, PositionTick, Tick},
    invariant::{CalculateSwapResult, QuoteResult, SwapHop},
    math::{
        liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice,
        token_amount::TokenAmount,
    },
    InvariantError,
};
use alloc::vec::Vec;
use ink::primitives::AccountId;

#[ink::trait_definition]
pub trait InvariantTrait {
    /// Retrieves the protocol fee represented as a percentage.
    #[ink(message)]
    fn get_protocol_fee(&self) -> Percentage;

    /// Allows an fee receiver to withdraw collected fees.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    ///
    /// # Errors
    /// - Reverts the call when the caller is an unauthorized receiver.
    ///
    /// # External contracts
    /// - PSP22
    #[ink(message)]
    fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError>;

    /// Allows an admin to adjust the protocol fee.
    ///
    /// # Parameters
    /// - `protocol_fee`: The expected fee represented as a percentage.
    ///
    /// # Errors
    /// - Reverts the call when the caller is an unauthorized user.
    #[ink(message)]
    fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Result<(), InvariantError>;

    /// Allows admin to change current fee receiver.
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
    /// - Fails if pool does not exist
    ///
    /// # External contracts
    /// - PSP22
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
    /// - `sqrt_price_limit`: A square root of price limit allowing the price to move for the swap to occur.
    ///
    /// # Events
    /// - On a successful swap, emits a `Swap` event for the freshly made swap.
    /// - On a successful swap, emits a `Cross Tick` event for every single tick crossed.
    ///
    /// # Errors
    /// - Fails if the user attempts to perform a swap with zero amounts.
    /// - Fails if the price has reached the specified price limit (or price associated with specified square root of price).
    /// - Fails if the user would receive zero tokens.
    /// - Fails if the allowance is insufficient or the user balance transfer fails.
    /// - Fails if there is insufficient liquidity in pool
    /// - Fails if pool does not exist
    ///
    /// # External contracts
    /// - PSP22
    #[ink(message)]
    fn swap(
        &mut self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError>;

    /// Performs atomic swap involving several pools based on the provided parameters.
    ///
    /// # Parameters
    /// - `amount_in`: The amount of tokens that the user wants to swap.
    /// - `expected_amount_out`: The amount of tokens that the user wants to receive as a result of the swaps.
    /// - `slippage`: The max acceptable percentage difference between the expected and actual amount of output tokens in a trade, not considering square root of target price as in the case of a swap.
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
    /// - Fails if pool does not exist
    ///
    /// # External contracts
    /// - PSP22
    #[ink(message)]
    fn swap_route(
        &mut self,
        amount_in: TokenAmount,
        expected_amount_out: TokenAmount,
        slippage: Percentage,
        swaps: Vec<SwapHop>,
    ) -> Result<(), InvariantError>;

    /// Simulates the swap without its execution.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    /// - `x_to_y`: A boolean specifying the swap direction.
    /// - `amount`: The amount of tokens that the user wants to swap.
    /// - `by_amount_in`: A boolean specifying whether the user provides the amount to swap or expects the amount out.
    /// - `sqrt_price_limit`: A square root of price limit allowing the price to move for the swap to occur.
    ///
    /// # Errors
    /// - Fails if the user attempts to perform a swap with zero amounts.
    /// - Fails if the price has reached the specified limit.
    /// - Fails if the user would receive zero tokens.
    /// - Fails if pool does not exist
    #[ink(message)]
    fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<QuoteResult, InvariantError>;

    /// Simulates multiple swaps without its execution.
    ///
    /// # Parameters
    /// - `amount_in`: The amount of tokens that the user wants to swap.
    /// - `swaps`: A vector containing all parameters needed to identify separate swap steps.
    ///
    /// # Errors
    /// - Fails if the user attempts to perform a swap with zero amounts.
    /// - Fails if the user would receive zero tokens.
    /// - Fails if pool does not exist
    #[ink(message)]
    fn quote_route(
        &mut self,
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
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
    /// - `owner_id`: An `AccountId` identifying the user who owns the position.
    /// - `index`: The index of the user position.
    ///
    /// # Errors
    /// - Fails if position cannot be found
    #[ink(message)]
    fn get_position(&mut self, owner_id: AccountId, index: u32)
        -> Result<Position, InvariantError>;

    /// Retrieves a vector containing all positions held by the user.
    ///
    /// # Parameters
    /// - `owner_id`: An `AccountId` identifying the user who owns the positions.
    #[ink(message)]
    fn get_all_positions(&mut self, owner_id: AccountId) -> Vec<Position>;

    /// Allows an authorized user (owner of the position) to claim collected fees.
    ///
    /// # Parameters
    /// - `index`: The index of the user position from which fees will be claimed.
    ///
    /// # Errors
    /// - Fails if the position cannot be found.
    ///
    /// # External contracts
    /// - PSP22
    #[ink(message)]
    fn claim_fee(&mut self, index: u32) -> Result<(TokenAmount, TokenAmount), InvariantError>;

    /// Removes a position. Sends tokens associated with specified position to the owner.
    ///
    /// # Parameters
    /// - `index`: The index of the user position to be removed.
    ///
    /// # Events
    /// - Emits a `Remove Position` event upon success.
    ///
    /// # Errors
    /// - Fails if Position cannot be found
    ///
    /// # External contracts
    /// - PSP22
    #[ink(message)]
    fn remove_position(&mut self, index: u32)
        -> Result<(TokenAmount, TokenAmount), InvariantError>;

    /// Allows admin to add a custom fee tier.
    ///
    /// # Parameters
    /// - `fee_tier`: A struct identifying the pool fee and tick spacing.
    ///
    /// # Errors
    /// - Fails if an unauthorized user attempts to create a fee tier.
    /// - Fails if the tick spacing is invalid.
    /// - Fails if the fee tier already exists.
    /// - Fails if fee is invalid
    #[ink(message)]
    fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;

    /// Query of whether the fee tier exists.
    ///
    /// # Parameters
    /// - `fee_tier`: A struct identifying the pool fee and tick spacing.
    #[ink(message)]
    fn fee_tier_exist(&self, fee_tier: FeeTier) -> bool;

    /// Removes an existing fee tier.
    ///
    /// # Parameters
    /// - `fee_tier`: A struct identifying the pool fee and tick spacing.
    ///
    /// # Errors
    /// - Fails if an unauthorized user attempts to remove a fee tier.
    /// - Fails if fee tier does not exist
    #[ink(message)]
    fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError>;

    /// Allows a user to create a custom pool on a specified token pair and fee tier.
    /// The contract specifies the order of tokens as x and y, the lower token address assigned as token x.
    /// The choice is deterministic.
    ///
    /// # Parameters
    /// - `token_0`: The address of the first token.
    /// - `token_1`: The address of the second token.
    /// - `fee_tier`: A struct identifying the pool fee and tick spacing.
    /// - `init_sqrt_price`: The square root of the price for the initial pool related to `init_tick`.
    /// - `init_tick`: The initial tick at which the pool will be created.
    ///
    /// # Errors
    /// - Fails if the specified fee tier cannot be found.
    /// - Fails if the user attempts to create a pool for the same tokens.
    /// - Fails if Pool with same tokens and fee tier already exist.
    /// - Fails if the init tick is not divisible by the tick spacing.
    /// - Fails if the init sqrt price is not related to the init tick.
    #[ink(message)]
    fn create_pool(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        fee_tier: FeeTier,
        init_sqrt_price: SqrtPrice,
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
    /// - Fails if there is no pool associated with created key
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
    ///
    /// # Errors
    /// - Fails if tick cannot be found
    #[ink(message)]
    fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError>;

    /// Checks if the tick at a specified index is initialized.
    ///
    /// # Parameters
    /// - `key`: A unique key that identifies the specified pool.
    /// - `index`: The tick index in the tickmap.
    #[ink(message)]
    fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool;

    /// Retrieves listed pools
    /// - `size`: Amount of pool keys to retrive
    /// - `offset`: The offset from which retrive pools.
    #[ink(message)]
    fn get_pools(&self, size: u8, offset: u16) -> Result<Vec<PoolKey>, InvariantError>;

    /// Retrieves available fee tiers
    #[ink(message)]
    fn get_fee_tiers(&self) -> Vec<FeeTier>;

    /// Retrieves list of lower and upper ticks of user positions.
    ///
    /// # Parameters
    /// - `owner`: An `AccountId` identifying the user who owns the position.
    /// - `offset`: The offset from the current position index.
    #[ink(message)]
    fn get_position_ticks(&self, owner: AccountId, offset: u32) -> Vec<PositionTick>;

    /// Retrieves the amount of positions held by the user.
    ///
    /// # Parameters
    /// - `owner`: An `AccountId` identifying the user who owns the position.
    #[ink(message)]
    fn get_user_position_amount(&self, owner: AccountId) -> u32;

    /// Retrieves indexes of initialized tickmap chunks
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    #[ink(message)]
    fn get_initialized_chunks(&self, pool_key: PoolKey) -> Vec<u16>;

    /// Retrieves tickmap chunks
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    #[ink(message)]
    fn get_tickmap(
        &self,
        pool_key: PoolKey,
        current_tick_index: i32,
        offset: u16,
        amount: u32,
    ) -> Vec<(u16, u64)>;

    /// Retrieves ticks of a specified pool.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    /// - `offset`: The offset from which ticks will be retrieved.
    #[ink(message)]
    fn get_liquidity_ticks(&self, pool_key: PoolKey, offset: u16) -> Vec<LiquidityTick>;

    /// Retrieves the amount of liquidity ticks of a specified pool.
    ///
    /// # Parameters
    /// - `pool_key`: A unique key that identifies the specified pool.
    #[ink(message)]
    fn get_liquidity_ticks_amount(&self, pool_key: PoolKey) -> u32;
}
