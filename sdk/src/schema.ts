import { CreatePositionEvent, CrossTickEvent, RemovePositionEvent, SwapEvent } from 'wasm/wasm.js'

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
  ChangeProtocolFee = 'invariantTrait::changeProtocolFee',
  CreatePool = 'invariantTrait::createPool',
  CreatePosition = 'invariantTrait::createPosition',
  TransferPosition = 'invariantTrait::transferPosition',
  RemovePosition = 'invariantTrait::removePosition',
  ClaimFee = 'invariantTrait::claimFee',
  AddFeeTier = 'invariantTrait::addFeeTier',
  RemoveFeeTier = 'invariantTrait::removeFeeTier',
  ChangeFeeReceiver = 'invariantTrait::changeFeeReceiver',
  WithdrawProtocolFee = 'invariantTrait::withdrawProtocolFee',
  Swap = 'invariantTrait::swap',
  SwapRoute = 'invariantTrait::swapRoute'
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

export enum InvariantEvent {
  CreatePositionEvent = 'CreatePositionEvent',
  CrossTickEvent = 'CrossTickEvent',
  RemovePositionEvent = 'RemovePositionEvent',
  SwapEvent = 'SwapEvent'
}

export type InvariantEventType =
  | CreatePositionEvent
  | CrossTickEvent
  | RemovePositionEvent
  | SwapEvent

export interface TxResultInterface<T> {
  hash: string
  events: T
}

export type TxResult = TxResultInterface<InvariantEventType[]>
export type CreatePositionTxResult = TxResultInterface<[CreatePositionEvent]>
export type RemovePositionTxResult = TxResultInterface<[RemovePositionEvent]>
export type SwapTxResult = TxResultInterface<[CrossTickEvent, SwapEvent] | [SwapEvent]>
export type SwapRouteTxResult = TxResultInterface<(CrossTickEvent | SwapEvent)[]>

export type ContractOptions = {
  storageDepositLimit: number | null
  refTime: number
  proofSize: number
}
