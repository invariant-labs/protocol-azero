#[ink(event)]
pub struct RemovePositionEvent {
    #[ink(topic)]
    timestamp: u64,
    address: AccountId,
    pool: PoolKey,
    liquidity: Liquidity,
    lower_tick: i32,
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
}
