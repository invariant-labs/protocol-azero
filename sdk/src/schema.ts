export enum InvariantQuery {
  ProtocolFee = 'invariantTrait::getProtocolFee',
  GetFeeTiers = 'invariantTrait::getFeeTiers',
  FeeTierExist = 'invariantTrait::feeTierExist'
}

export enum InvariantTx {
  ChangeProtocolFee = 'invariantTrait::changeProtocolFee',
  AddFeeTier = 'invariantTrait::addFeeTier',
  RemoveFeeTier = 'invariantTrait::removeFeeTier',
  ChangeFeeReceiver = 'invariantTrait::changeFeeReceiver',
  WithdrawProtocolFee = 'invariantTrait::withdrawProtocolFee'
}

export enum PSP22Query {
  TokenName = 'psp22Metadata::tokenName',
  TokenSymbol = 'psp22Metadata::tokenSymbol',
  TokenDecimals = 'psp22Metadata::tokenDecimals',
  BalanceOf = 'psp22::balanceOf',
  TotalSupply = 'psp22::totalSupply',
  Allowance = 'psp22::allowance'
}

export enum PSP22Tx {
  Mint = 'psp22Mintable::mint',
  Transfer = 'psp22::transfer',
  Approve = 'psp22::approve'
}

export enum WrappedAZEROTx {
  Deposit = 'wrappedAZERO::deposit',
  Withdraw = 'wrappedAZERO::withdraw'
}

// TODO: replace this class
export class FeeTier {
  fee: { v: bigint }
  tickSpacing: bigint

  constructor(fee: bigint, tickSpacing: bigint) {
    this.fee = { v: fee }
    this.tickSpacing = tickSpacing
  }
}

// TODO: replace this class
export class PoolKey {
  token0: string
  token1: string
  fee_tier: FeeTier

  constructor(token0: string, token1: string, fee_tier: FeeTier) {
    this.token0 = token0
    this.token1 = token1
    this.fee_tier = fee_tier
  }
}

// TODO: replace this class
export class Type {
  v: bigint

  constructor(v: bigint) {
    this.v = v
  }
}
