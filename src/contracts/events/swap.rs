#[ink(event)]
pub struct SwapEvent {
    #[ink(topic)]
    timestamp: u64,
    address: AccountId,
    pool: PoolKey,
    amount_in: TokenAmount,
    amount_out: TokenAmount,
    fee: TokenAmount,
    start_sqrt_price: SqrtPrice,
    target_sqrt_price: SqrtPrice,
}
