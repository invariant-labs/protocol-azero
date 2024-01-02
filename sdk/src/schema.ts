export enum InvariantQuery {
  ProtocolFee = 'invariantTrait::getProtocolFee',
  GetPool = 'invariantTrait::getPool',
  GetPools = 'invariantTrait::getPools'
}

export enum InvariantTx {
  ChangeProtocolFee = 'invariantTrait::changeProtocolFee',
  createPool = 'invariantTrait::createPool'
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
