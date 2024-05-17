/* eslint camelcase: off */

import {
  FeeTier,
  InvariantError,
  Liquidity,
  LiquidityTick,
  Percentage,
  Pool,
  PoolKey,
  Position,
  PositionTick,
  QuoteResult,
  SqrtPrice,
  SwapHop,
  Tick,
  TokenAmount,
  calculateTick,
  getMaxSqrtPrice,
  getMinSqrtPrice
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { ApiPromise } from '@polkadot/api'
import { Abi, ContractPromise } from '@polkadot/api-contract'
import { SubmittableExtrinsic } from '@polkadot/api/promise/types'
import { Bytes } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { deployContract } from '@scio-labs/use-inkathon'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME } from './consts.js'
import { Network } from './network.js'
import {
  ContractOptions,
  CreatePositionTxResult,
  InvariantEvent,
  InvariantQuery,
  InvariantTx,
  RemovePositionTxResult,
  SwapRouteTxResult,
  SwapTxResult,
  TxResult
} from './schema.js'
import {
  calculateSqrtPriceAfterSlippage,
  constructTickmap,
  createSignAndSendTx,
  createTx,
  getAbi,
  getDeploymentData,
  parse,
  parseEvent,
  sendQuery
} from './utils.js'
import { Tickmap } from './wasm/pkg/invariant_a0_wasm.js'

export class Invariant {
  contract: ContractPromise
  api: ApiPromise
  gasLimit: WeightV2
  storageDepositLimit: number | null
  waitForFinalization: boolean
  abi: Abi
  eventListeners: { identifier: InvariantEvent; listener: (event: any) => void }[] = []
  eventListenerApiStarted: boolean = false

  private constructor(
    api: ApiPromise,
    network: Network,
    abi: any,
    address: string,
    storageDepositLimit: number | null = null,
    refTime: number = DEFAULT_REF_TIME,
    proofSize: number = DEFAULT_PROOF_SIZE
  ) {
    this.api = api
    this.waitForFinalization = network !== Network.Local
    this.contract = new ContractPromise(this.api, abi, address)
    this.gasLimit = api.registry.createType('WeightV2', {
      refTime,
      proofSize
    }) as WeightV2
    this.storageDepositLimit = storageDepositLimit
    this.abi = new Abi(abi)
  }

  static async deploy(
    api: ApiPromise,
    network: Network,
    deployer: IKeyringPair,
    fee: Percentage = 0n,
    options?: ContractOptions
  ): Promise<Invariant> {
    const deploymentData = await getDeploymentData('invariant')
    const deploy = await deployContract(
      api,
      deployer,
      deploymentData.abi,
      deploymentData.wasm,
      'new',
      [fee]
    )

    return new Invariant(
      api,
      network,
      deploymentData.abi,
      deploy.address,
      options?.storageDepositLimit,
      options?.refTime,
      options?.proofSize
    )
  }

  static async load(
    api: ApiPromise,
    network: Network,
    address: string,
    options?: ContractOptions
  ): Promise<Invariant> {
    const abi = await getAbi('invariant')

    return new Invariant(
      api,
      network,
      abi,
      address,
      options?.storageDepositLimit,
      options?.refTime,
      options?.proofSize
    )
  }

  on(identifier: InvariantEvent, listener: (event: any) => void): void {
    if (!this.eventListenerApiStarted) {
      this.eventListenerApiStarted = true

      this.api.query.system.events((events: any) => {
        if (this.eventListeners.length !== 0) {
          events.forEach((record: any) => {
            const { event } = record

            if (!this.api.events.contracts.ContractEmitted.is(event)) {
              return
            }

            const [account_id, contract_evt] = event.data

            if (account_id.toString() !== this.contract?.address.toString()) {
              return
            }

            const decoded = this.abi.decodeEvent(contract_evt as Bytes)

            if (!decoded) {
              return
            }

            const parsedEvent = parseEvent(decoded)

            this.eventListeners.map(eventListener => {
              if (eventListener.identifier === decoded.event.identifier) {
                eventListener.listener(parsedEvent)
              }
            })
          })
        }
      })
    }

    this.eventListeners.push({ identifier, listener })
  }

  off(identifier: InvariantEvent, listener?: (event: any) => void): void {
    this.eventListeners = this.eventListeners.filter(eventListener => {
      if (listener) {
        return !(identifier === eventListener.identifier && listener === eventListener.listener)
      } else {
        return !(identifier === eventListener.identifier)
      }
    })
  }

  async getProtocolFee(): Promise<Percentage> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.ProtocolFee,
      []
    )
  }

  changeProtocolFeeTx(fee: Percentage): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.ChangeProtocolFee,
      [fee]
    )
  }

  async changeProtocolFee(
    account: IKeyringPair,
    fee: Percentage,
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.ChangeProtocolFee,
      [fee],
      this.waitForFinalization,
      block
    )
  }

  addFeeTierTx(feeTier: FeeTier): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.AddFeeTier,
      [feeTier]
    )
  }

  async addFeeTier(
    account: IKeyringPair,
    feeTier: FeeTier,
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.AddFeeTier,
      [feeTier],
      this.waitForFinalization,
      block
    )
  }

  removeFeeTierTx(feeTier: FeeTier): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.RemoveFeeTier,
      [feeTier]
    )
  }

  async removeFeeTier(
    account: IKeyringPair,
    feeTier: FeeTier,
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.RemoveFeeTier,
      [feeTier],
      this.waitForFinalization,
      block
    )
  }

  async getFeeTiers(): Promise<FeeTier[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetFeeTiers,
      []
    )
  }

  async feeTierExist(feeTier: FeeTier): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.FeeTierExist,
      [feeTier]
    )
  }

  changeFeeReceiverTx(poolKey: PoolKey, feeReceiver: string): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.ChangeFeeReceiver,
      [poolKey, feeReceiver]
    )
  }

  async changeFeeReceiver(
    account: IKeyringPair,
    poolKey: PoolKey,
    feeReceiver: string,
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.ChangeFeeReceiver,
      [poolKey, feeReceiver],
      this.waitForFinalization,
      block
    )
  }

  withdrawProtocolFeeTx(poolKey: PoolKey): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.WithdrawProtocolFee,
      [poolKey]
    )
  }

  async withdrawProtocolFee(
    account: IKeyringPair,
    poolKey: PoolKey,
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.WithdrawProtocolFee,
      [poolKey],
      this.waitForFinalization,
      block
    )
  }

  async getPosition(owner: string, index: bigint): Promise<Position> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetPosition,
      [owner, index]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getPositions(owner: string): Promise<Position[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetAllPositions,
      [owner]
    )
  }

  createPositionTx(
    poolKey: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: Liquidity,
    spotSqrtPrice: SqrtPrice,
    slippageTolerance: Percentage
  ): SubmittableExtrinsic {
    const slippageLimitLower = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      true
    )
    const slippageLimitUpper = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      false
    )

    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.CreatePosition,
      [poolKey, lowerTick, upperTick, liquidityDelta, slippageLimitLower, slippageLimitUpper]
    )
  }

  async createPosition(
    account: IKeyringPair,
    poolKey: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: Liquidity,
    spotSqrtPrice: SqrtPrice,
    slippageTolerance: Percentage,
    block: boolean = true
  ): Promise<CreatePositionTxResult> {
    const slippageLimitLower = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      true
    )
    const slippageLimitUpper = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      false
    )

    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.CreatePosition,
      [poolKey, lowerTick, upperTick, liquidityDelta, slippageLimitLower, slippageLimitUpper],
      this.waitForFinalization,
      block
    ) as Promise<CreatePositionTxResult>
  }

  transferPositionTx(index: bigint, receiver: string): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.TransferPosition,
      [index, receiver]
    )
  }

  async transferPosition(
    account: IKeyringPair,
    index: bigint,
    receiver: string,
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.TransferPosition,
      [index, receiver],
      this.waitForFinalization,
      block
    )
  }

  removePositionTx(index: bigint): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.RemovePosition,
      [index]
    )
  }

  async removePosition(
    account: IKeyringPair,
    index: bigint,
    block: boolean = true
  ): Promise<RemovePositionTxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.RemovePosition,
      [index],
      this.waitForFinalization,
      block
    ) as Promise<RemovePositionTxResult>
  }

  claimFeeTx(index: bigint): SubmittableExtrinsic {
    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.ClaimFee,
      [index]
    )
  }

  async claimFee(account: IKeyringPair, index: bigint, block: boolean = true): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.ClaimFee,
      [index],
      this.waitForFinalization,
      block
    )
  }

  async getTick(key: PoolKey, index: bigint): Promise<Tick> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetTick,
      [key, index]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async isTickInitialized(key: PoolKey, index: bigint): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.IsTickInitialized,
      [key, index]
    )
  }

  async getPool(token0: string, token1: string, feeTier: FeeTier): Promise<Pool> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetPool,
      [token0, token1, feeTier]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getPoolKeys(size: bigint, offset: bigint): Promise<PoolKey[]> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetPools,
      [size, offset]
    )
    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  createPoolTx(poolKey: PoolKey, initSqrtPrice: SqrtPrice): SubmittableExtrinsic {
    const initTick = calculateTick(initSqrtPrice, poolKey.feeTier.tickSpacing)

    return createTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      InvariantTx.CreatePool,
      [poolKey.tokenX, poolKey.tokenY, poolKey.feeTier, initSqrtPrice, initTick]
    )
  }

  async createPool(
    account: IKeyringPair,
    poolKey: PoolKey,
    initSqrtPrice: SqrtPrice,
    block: boolean = true
  ): Promise<TxResult> {
    const initTick = calculateTick(initSqrtPrice, poolKey.feeTier.tickSpacing)

    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.CreatePool,
      [poolKey.tokenX, poolKey.tokenY, poolKey.feeTier, initSqrtPrice, initTick],
      this.waitForFinalization,
      block
    )
  }

  async quote(
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean
  ): Promise<QuoteResult> {
    const sqrtPriceLimit: SqrtPrice = xToY
      ? getMinSqrtPrice(poolKey.feeTier.tickSpacing)
      : getMaxSqrtPrice(poolKey.feeTier.tickSpacing)

    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.Quote,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async quoteRoute(amountIn: TokenAmount, swaps: SwapHop[]): Promise<TokenAmount> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.QuoteRoute,
      [amountIn, swaps]
    )
  }

  swapTx(
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice
  ): SubmittableExtrinsic {
    return createTx(this.contract, this.gasLimit, this.storageDepositLimit, 0n, InvariantTx.Swap, [
      poolKey,
      xToY,
      amount,
      byAmountIn,
      sqrtPriceLimit
    ])
  }

  async swap(
    account: IKeyringPair,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice,
    block: boolean = true
  ): Promise<SwapTxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.Swap,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit],
      this.waitForFinalization,
      block
    ) as Promise<SwapTxResult>
  }

  async swapRoute(
    account: IKeyringPair,
    amountIn: TokenAmount,
    expectedAmountOut: TokenAmount,
    slippage: Percentage,
    swaps: SwapHop[],
    block: boolean = true
  ): Promise<SwapRouteTxResult> {
    return createSignAndSendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.SwapRoute,
      [amountIn, expectedAmountOut, slippage, swaps],
      this.waitForFinalization,
      block
    ) as Promise<SwapRouteTxResult>
  }

  async getPositionTicks(owner: string, offset: bigint): Promise<PositionTick[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.getPositionTicks,
      [owner, offset]
    )
  }

  async getRawTickmap(
    userAddress: string,
    poolKey: PoolKey,
    currentTickIndex: bigint
  ): Promise<Tickmap> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetTickmap,
      [poolKey, currentTickIndex],
      userAddress
    )
    
    return {
        bitmap: new Map<bigint, bigint>(result)
    } 
  }

  async getFullTickmap(
    userAddress: string,
    poolKey: PoolKey,
    startingTickIndex: bigint
  ): Promise<Tickmap> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetTickmap,
      [poolKey, 1n],
      userAddress
    )
    
    return {
        bitmap: new Map<bigint, bigint>(result)
    } 
  }




  async getTickmap(poolKey: PoolKey, currentTickIndex: bigint): Promise<bigint[]> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.GetTickmap,
      [poolKey, currentTickIndex]
    )
    return constructTickmap(result, poolKey.feeTier.tickSpacing)
  }



  async getLiquidityTicks(poolKey: PoolKey, offset: bigint): Promise<LiquidityTick[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.getLiquidityTicks,
      [poolKey, offset]
    )
  }

  async getUserPositionAmount(owner: string): Promise<bigint> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.getUserPositionAmount,
      [owner]
    )
  }

  async getLiquidityTicksAmount(poolKey: PoolKey): Promise<bigint> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      InvariantQuery.getLiquidityTicksAmount,
      [poolKey]
    )
  }
}
