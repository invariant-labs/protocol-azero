use crate::contracts::PoolKey;
use crate::math::{liquidity::Liquidity, sqrt_price::sqrt_price::SqrtPrice};
use ink::primitives::AccountId;

#[ink(event)]
pub enum InvariantEvents {
    CreatePositionEvent {
        // #[ink(topic)]
        timestamp: u64,
        address: AccountId,
        pool: PoolKey,
        liquidity: Liquidity,
        lower_tick: i32,
        upper_tick: i32,
        current_sqrt_price: SqrtPrice,
    },
}
