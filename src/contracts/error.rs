#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum InvariantError {
    NotAdmin,
    NotFeeReceiver,
    PoolAlreadyExist,
    PoolNotFound,
    TickAlreadyExist,
    InvalidTickIndexOrTickSpacing,
    PositionNotFound,
    TickNotFound,
    FeeTierNotFound,
    PoolKeyNotFound,
    AmountIsZero,
    WrongLimit,
    PriceLimitReached,
    NoGainSwap,
    InvalidTickSpacing,
    FeeTierAlreadyExist,
    PoolKeyAlreadyExist,
    UnauthorizedFeeReceiver,
    ZeroLiquidity,
    TransferError,
    TokensAreSame,
    AmountUnderMinimumAmountOut,
    InvalidFee,
    NotEmptyTickDeinitialization,
    InvalidInitTick,
    InvalidInitSqrtPrice,
    InvalidSize,
    InvalidTickIndex,
    TickLimitReached,
    AddOverflow(u128, u128),
    SubUnderflow(u128, u128),
    MulOverflow,
    DivByZero,
    WAZEROWithdrawError,
    SetCodeHashError,
}
