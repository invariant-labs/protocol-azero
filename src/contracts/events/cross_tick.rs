#[ink(event)]
pub struct CrossTickEvent {
    #[ink(topic)]
    timestamp: u64,
    address: AccountId,
    pool: PoolKey,
    index: i32,
}
