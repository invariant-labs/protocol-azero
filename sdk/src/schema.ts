import {
  CreatePositionEvent,
  CrossTickEvent,
  Liquidity,
  RemovePositionEvent,
  SwapEvent
} from '../src/wasm/pkg/invariant_a0_wasm.js'

const invariantActionPrefix = 'invariantTrait::'

export enum InvariantQuery {
  ProtocolFee = 'invariantTrait::getProtocolFee',
  GetFeeTiers = 'invariantTrait::getFeeTiers',
  FeeTierExist = 'invariantTrait::feeTierExist',
  GetPool = 'invariantTrait::getPool',
  GetPools = 'invariantTrait::getPools',
  GetTick = 'invariantTrait::getTick',
  IsTickInitialized = 'invariantTrait::isTickInitialized',
  GetPosition = 'invariantTrait::getPosition',
  GetAllPositions = 'invariantTrait::getAllPositions',
  Quote = 'invariantTrait::quote',
  QuoteRoute = 'invariantTrait::quoteRoute',
  getPositionTicks = 'invariantTrait::getPositionTicks',
  getUserPositionAmount = 'invariantTrait::getUserPositionAmount',
  GetTickmap = 'invariantTrait::getTickmap',
  getLiquidityTicks = 'invariantTrait::getLiquidityTicks',
  getLiquidityTicksAmount = 'invariantTrait::getLiquidityTicksAmount'
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
  SwapRoute = `${invariantActionPrefix}swapRoute`
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

export type CreatePositionTxResult = EventTxResult<[CreatePositionEvent]>
export type RemovePositionTxResult = EventTxResult<[RemovePositionEvent]>
export type SwapTxResult = EventTxResult<[CrossTickEvent, SwapEvent] | [SwapEvent]>
export type SwapRouteTxResult = EventTxResult<(CrossTickEvent | SwapEvent)[]>

export type ContractOptions = {
  storageDepositLimit: number | null
  refTime: number
  proofSize: number
}

export interface LiquidityBreakpoint {
  liquidity: Liquidity
  index: bigint
}
