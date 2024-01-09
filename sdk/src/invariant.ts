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
  TokenAmount
} from 'math/math.js'
import { Network } from './network.js'
import { InvariantEvent, InvariantQuery, InvariantTx, TxResult } from './schema.js'
import {
  DEFAULT_PROOF_SIZE,
  DEFAULT_REF_TIME,
  convertArr,
  convertObj,
  getDeploymentData,
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
              eventListener.listener(convertObj(eventObj))
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
    ) as Promise<Percentage>
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
      0,
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
      0,
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
      0,
      account,
      InvariantTx.RemoveFeeTier,
      [feeTier],
      this.waitForFinalization,
      block
    )
  }

  async getFeeTiers(account: IKeyringPair): Promise<FeeTier[]> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetFeeTiers,
      []
    )) as any

    return convertArr(result)
  }

  async feeTierExist(account: IKeyringPair, feeTier: FeeTier): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.FeeTierExist,
      [feeTier]
    ) as Promise<boolean>
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
      0,
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
      0,
      account,
      InvariantTx.WithdrawProtocolFee,
      [poolKey],
      this.waitForFinalization,
      block
    )
  }

  async getPosition(account: IKeyringPair, owner: string, index: bigint): Promise<Position> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPosition,
      [owner, index]
    )) as any

    if (result.ok) {
      return convertObj(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getPositions(account: IKeyringPair, owner: string): Promise<Position[]> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetAllPositions,
      [owner]
    )) as any

    return convertArr(result)
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
  ): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.CreatePosition,
      [poolKey, lowerTick, upperTick, liquidityDelta, slippageLimitLower, slippageLimitUpper],
      this.waitForFinalization,
      block
    )
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
      0,
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
  ): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.RemovePosition,
      [index],
      this.waitForFinalization,
      block
    )
  }

  async claimFee(account: IKeyringPair, index: bigint, block: boolean = true): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.ClaimFee,
      [index],
      this.waitForFinalization,
      block
    )
  }

  async getTick(account: IKeyringPair, key: PoolKey, index: bigint): Promise<Tick> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetTick,
      [key, index]
    )) as any

    if (result.ok) {
      return convertObj(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async isTickInitialized(account: IKeyringPair, key: PoolKey, index: bigint): Promise<boolean> {
    return (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.IsTickInitialized,
      [key, index]
    )) as Promise<boolean>
  }

  async getPool(
    account: IKeyringPair,
    token0: string,
    token1: string,
    feeTier: FeeTier
  ): Promise<Pool> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPool,
      [token0, token1, feeTier]
    )) as any

    if (result.ok) {
      return convertObj(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getPools(account: IKeyringPair): Promise<Pool[]> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPools,
      []
    )) as any

    return convertArr(result)
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
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
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
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.Quote,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit]
    )) as any

    return convertObj(result)
  }

  async quoteRoute(
    account: IKeyringPair,
    amountIn: TokenAmount,
    swaps: SwapHop[]
  ): Promise<TokenAmount> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.QuoteRoute,
      [amountIn, swaps]
    )) as any

    return convertObj(result)
  }

  async swap(
    account: IKeyringPair,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.Swap,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit],
      this.waitForFinalization,
      block
    )
  }

  async swapRoute(
    account: IKeyringPair,
    amountIn: TokenAmount,
    expectedAmountOut: TokenAmount,
    slippage: Percentage,
    swaps: SwapHop[],
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.SwapRoute,
      [amountIn, expectedAmountOut, slippage, swaps],
      this.waitForFinalization,
      block
    )
  }
}
