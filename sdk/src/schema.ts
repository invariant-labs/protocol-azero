import { Liquidity } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'

const invariantActionPrefix = 'invariantTrait::'

export enum InvariantQuery {
  ProtocolFee = `${invariantActionPrefix}getProtocolFee`,
  GetFeeTiers = `${invariantActionPrefix}getFeeTiers`,
  FeeTierExist = `${invariantActionPrefix}feeTierExist`,
  GetPool = `${invariantActionPrefix}getPool`,
  GetPoolKeys = `${invariantActionPrefix}getPoolKeys`,
  GetTick = `${invariantActionPrefix}getTick`,
  IsTickInitialized = `${invariantActionPrefix}isTickInitialized`,
  GetPosition = `${invariantActionPrefix}getPosition`,
  GetAllPositions = `${invariantActionPrefix}getAllPositions`,
  GetPositions = `${invariantActionPrefix}getPositions`,
  Quote = `${invariantActionPrefix}quote`,
  QuoteRoute = `${invariantActionPrefix}quoteRoute`,
  GetUserPositionAmount = `${invariantActionPrefix}getUserPositionAmount`,
  GetTickmap = `${invariantActionPrefix}getTickmap`,
  GetLiquidityTicks = `${invariantActionPrefix}getLiquidityTicks`,
  GetLiquidityTicksAmount = `${invariantActionPrefix}getLiquidityTicksAmount`,
  GetAllPoolsForPair = `${invariantActionPrefix}getAllPoolsForPair`
}

export enum InvariantTx {
  ChangeProtocolFee = `${invariantActionPrefix}changeProtocolFee`,
  CreatePool = `${invariantActionPrefix}createPool`,
  CreatePosition = `${invariantActionPrefix}createPosition`,
  TransferPosition = `${invariantActionPrefix}transferPosition`,
  RemovePosition = `${invariantActionPrefix}removePosition`,
  ClaimFee = `${invariantActionPrefix}claimFee`,
  AddFeeTier = `${invariantActionPrefix}addFeeTier`,
  RemoveFeeTier = `${invariantActionPrefix}removeFeeTier`,
  ChangeFeeReceiver = `${invariantActionPrefix}changeFeeReceiver`,
  WithdrawProtocolFee = `${invariantActionPrefix}withdrawProtocolFee`,
  Swap = `${invariantActionPrefix}swap`,
  SwapRoute = `${invariantActionPrefix}swapRoute`,
  WithdrawAllWAZERO = `${invariantActionPrefix}withdrawAllWazero`,
  SetCode = `${invariantActionPrefix}setCode`
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

export type Tx = InvariantTx | PSP22Tx | WrappedAZEROTx
export type Query = InvariantQuery | PSP22Query

const invariantEventPrefix = 'invariant::contracts::events::'

export enum InvariantEvent {
  CreatePositionEvent = `${invariantEventPrefix}CreatePositionEvent`,
  CrossTickEvent = `${invariantEventPrefix}CrossTickEvent`,
  RemovePositionEvent = `${invariantEventPrefix}RemovePositionEvent`,
  SwapEvent = `${invariantEventPrefix}SwapEvent`
}

export type TxResult = {
  hash: string
}

export interface EventTxResult<T> extends TxResult {
  events: T
}

export type CreatePositionTxResult = EventTxResult<object[]>
export type RemovePositionTxResult = EventTxResult<object[]>
export type SwapTxResult = EventTxResult<object[]>
export type SwapRouteTxResult = EventTxResult<object[]>

export type ContractOptions = {
  storageDepositLimit: number | null
  refTime: number
  proofSize: number
}

export interface LiquidityBreakpoint {
  liquidity: Liquidity
  index: bigint
}
