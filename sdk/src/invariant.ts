/* eslint camelcase: off */

import { ApiPromise } from '@polkadot/api'
import { Abi, ContractPromise } from '@polkadot/api-contract'
import { Bytes } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import {
  FeeTier,
  InvariantError,
  Liquidity,
  Percentage,
  Pool,
  PoolKey,
  Position,
  QuoteResult,
  SqrtPrice,
  SwapHop,
  Tick,
  TokenAmount,
  checkTickToSqrtPriceRelationship
} from 'math/math.js'
import { Network } from './network.js'
import {
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
  DEFAULT_PROOF_SIZE,
  DEFAULT_REF_TIME,
  getDeploymentData,
  parse,
  sendQuery,
  sendTx
} from './utils.js'

export class Invariant {
  contract: ContractPromise
  api: ApiPromise
  gasLimit: WeightV2
  storageDepositLimit: number | null
  waitForFinalization: boolean
  abi: Abi | null = null
  eventListeners: { identifier: InvariantEvent; listener: (event: any) => void }[] = []

  private constructor(
    api: ApiPromise,
    network: Network,
    storageDepositLimit: number | null = null,
    refTime: number = DEFAULT_REF_TIME,
    proofSize: number = DEFAULT_PROOF_SIZE,
    abi: any,
    deploymentAddress: string
  ) {
    this.api = api
    this.gasLimit = api.registry.createType('WeightV2', {
      refTime,
      proofSize
    }) as WeightV2
    this.storageDepositLimit = storageDepositLimit
    this.waitForFinalization = network != Network.Local
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
    this.abi = new Abi(abi)
  }

  static async getContract(
    api: ApiPromise,
    account: IKeyringPair,
    storageDepositLimit: number | null = null,
    refTime: number = DEFAULT_REF_TIME,
    proofSize: number = DEFAULT_PROOF_SIZE,
    initFee: Percentage,
    network: Network
  ): Promise<Invariant> {
    const invariantData = await getDeploymentData('invariant')
    if (process.env.INVARIANT_ADDRESS && network != Network.Local) {
      return new Invariant(
        api,
        network,
        storageDepositLimit,
        refTime,
        proofSize,
        invariantData.abi,
        process.env.INVARIANT_ADDRESS
      )
    } else {
      const invariantDeploy = await Invariant.deploy(
        api,
        account,
        invariantData.abi,
        invariantData.wasm,
        initFee
      )
      const invariant = new Invariant(
        api,
        Network.Local,
        storageDepositLimit,
        refTime,
        proofSize,
        invariantData.abi,
        invariantDeploy.address
      )
      return invariant
    }
  }

  static async deploy(
    api: ApiPromise,
    account: IKeyringPair,
    abi: any,
    wasm: Buffer,
    fee: Percentage
  ): Promise<DeployedContract> {
    return deployContract(api, account, abi, wasm, 'new', [fee])
  }

  on(identifier: InvariantEvent, listener: (event: any) => void): void {
    if (this.eventListeners.length === 0) {
      this.api.query.system.events((events: any) => {
        events.forEach((record: any) => {
          const { event } = record

          if (!this.api.events.contracts.ContractEmitted.is(event)) {
            return
          }

          const [account_id, contract_evt] = event.data

          if (account_id.toString() !== this.contract?.address.toString()) {
            return
          }

          const decoded = this.abi?.decodeEvent(contract_evt as Bytes)

          if (!decoded) {
            return
          }

          const eventObj: { [key: string]: any } = {}

          for (let i = 0; i < decoded.args.length; i++) {
            eventObj[decoded.event.args[i].name] = decoded.args[i].toPrimitive()
          }

          this.eventListeners.map(eventListener => {
            if (eventListener.identifier === decoded.event.identifier) {
              eventListener.listener(parse(eventObj))
            }
          })
        })
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

  async getProtocolFee(account: IKeyringPair): Promise<Percentage> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.ProtocolFee,
      []
    )
  }

  async changeProtocolFee(
    account: IKeyringPair,
    fee: Percentage,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
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

  async addFeeTier(
    account: IKeyringPair,
    feeTier: FeeTier,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
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

  async removeFeeTier(
    account: IKeyringPair,
    feeTier: FeeTier,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
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

  async getFeeTiers(account: IKeyringPair): Promise<FeeTier[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetFeeTiers,
      []
    )
  }

  async feeTierExist(account: IKeyringPair, feeTier: FeeTier): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.FeeTierExist,
      [feeTier]
    )
  }

  async changeFeeReceiver(
    account: IKeyringPair,
    poolKey: PoolKey,
    feeReceiver: string,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
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

  async withdrawProtocolFee(
    account: IKeyringPair,
    poolKey: PoolKey,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
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

  async getPosition(account: IKeyringPair, owner: string, index: bigint): Promise<Position> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPosition,
      [owner, index]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getPositions(account: IKeyringPair, owner: string): Promise<Position[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetAllPositions,
      [owner]
    )
  }

  async createPosition(
    account: IKeyringPair,
    poolKey: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: Liquidity,
    slippageLimitLower: SqrtPrice,
    slippageLimitUpper: SqrtPrice,
    block: boolean = true
  ): Promise<CreatePositionTxResult> {
    return sendTx(
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

  async transferPosition(
    account: IKeyringPair,
    index: bigint,
    receiver: string,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
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

  async removePosition(
    account: IKeyringPair,
    index: bigint,
    block: boolean = true
  ): Promise<RemovePositionTxResult> {
    return sendTx(
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

  async claimFee(account: IKeyringPair, index: bigint, block: boolean = true): Promise<TxResult> {
    return sendTx(
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

  async getTick(account: IKeyringPair, key: PoolKey, index: bigint): Promise<Tick> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetTick,
      [key, index]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async isTickInitialized(account: IKeyringPair, key: PoolKey, index: bigint): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.IsTickInitialized,
      [key, index]
    )
  }

  async getPool(
    account: IKeyringPair,
    token0: string,
    token1: string,
    feeTier: FeeTier
  ): Promise<Pool> {
    const result = await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPool,
      [token0, token1, feeTier]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getPools(account: IKeyringPair): Promise<Pool[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPools,
      []
    )
  }

  async createPool(
    account: IKeyringPair,
    token0: string,
    token1: string,
    feeTier: FeeTier,
    initSqrtPrice: SqrtPrice,
    initTick: bigint,
    block: boolean = true
  ): Promise<TxResult> {
    const isInRelationship = checkTickToSqrtPriceRelationship(
      initTick,
      feeTier.tickSpacing,
      initSqrtPrice
    )

    if (!isInRelationship) {
      throw new Error(InvariantError[24])
    }

    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      InvariantTx.CreatePool,
      [token0, token1, feeTier, initSqrtPrice, initTick],
      this.waitForFinalization,
      block
    )
  }

  async quote(
    account: IKeyringPair,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice
  ): Promise<QuoteResult> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.Quote,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit]
    )
  }

  async quoteRoute(
    account: IKeyringPair,
    amountIn: TokenAmount,
    swaps: SwapHop[]
  ): Promise<TokenAmount> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.QuoteRoute,
      [amountIn, swaps]
    )
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
    return sendTx(
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
    return sendTx(
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
  async getTickmap(
    account: IKeyringPair,
    poolKey: PoolKey,
    currentTickIndex: bigint
  ): Promise<bigint[][]> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetTickmap,
      [poolKey, currentTickIndex]
    )) as any
    return parse(result)
  }
}
