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
  QuoteResult,
  SqrtPrice,
  SwapHop,
  Tick,
  Tickmap,
  TokenAmount,
  calculateTick,
  getMaxSqrtPrice,
  getMinSqrtPrice
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { ApiPromise } from '@polkadot/api'
import { Abi, ContractPromise } from '@polkadot/api-contract'
import { SubmittableExtrinsic } from '@polkadot/api/types/submittable'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { deployContract } from '@scio-labs/use-inkathon'
import {
  CHUNK_SIZE,
  DEFAULT_PROOF_SIZE,
  DEFAULT_REF_TIME,
  LIQUIDITY_TICKS_LIMIT,
  MAX_POOL_KEYS_RETURNED,
  MAX_TICKMAP_QUERY_SIZE,
  POSITIONS_ENTRIES_LIMIT
} from './consts.js'
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
  assert,
  calculateSqrtPriceAfterSlippage,
  createSignAndSendTx,
  createTx,
  deserializeTickFromContract,
  deserializePoolFromContract,
  deserializePositionFromContract,
  extractError,
  getAbi,
  getDeploymentData,
  getMaxTick,
  getMinTick,
  integerSafeCast,
  parse,
  parseEvent,
  positionToTick,
  sendQuery
} from './utils.js'

type Page = { index: number; entries: [Position, Pool][] }

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

            const [account_id] = event.data

            if (account_id.toString() !== this.contract?.address.toString()) {
              return
            }

            const decoded = this.abi.decodeEvent(record)

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

  changeProtocolFeeTx(
    fee: Percentage,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.ChangeProtocolFee,
      [fee]
    )
  }

  async changeProtocolFee(
    account: IKeyringPair,
    fee: Percentage,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.ChangeProtocolFee,
      [fee],
      this.waitForFinalization,
      block
    )
  }

  addFeeTierTx(
    feeTier: FeeTier,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.AddFeeTier,
      [feeTier]
    )
  }

  async addFeeTier(
    account: IKeyringPair,
    feeTier: FeeTier,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.AddFeeTier,
      [feeTier],
      this.waitForFinalization,
      block
    )
  }

  removeFeeTierTx(
    feeTier: FeeTier,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.RemoveFeeTier,
      [feeTier]
    )
  }

  async removeFeeTier(
    account: IKeyringPair,
    feeTier: FeeTier,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
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

  async feeTierExist(
    feeTier: FeeTier,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.FeeTierExist,
      [feeTier]
    )
  }

  changeFeeReceiverTx(
    poolKey: PoolKey,
    feeReceiver: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.ChangeFeeReceiver,
      [poolKey, feeReceiver]
    )
  }

  async changeFeeReceiver(
    account: IKeyringPair,
    poolKey: PoolKey,
    feeReceiver: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.ChangeFeeReceiver,
      [poolKey, feeReceiver],
      this.waitForFinalization,
      block
    )
  }

  withdrawProtocolFeeTx(
    poolKey: PoolKey,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.WithdrawProtocolFee,
      [poolKey]
    )
  }

  async withdrawProtocolFee(
    account: IKeyringPair,
    poolKey: PoolKey,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.WithdrawProtocolFee,
      [poolKey],
      this.waitForFinalization,
      block
    )
  }

  async getPosition(
    owner: string,
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<Position> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetPosition,
      [owner, index]
    )

    if (result.ok) {
      return deserializePositionFromContract(parse(result.ok))
    } else {
      throw new Error(extractError(result.err))
    }
  }

  async getPositions(
    owner: string,
    size: bigint,
    offset: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<[[Position, Pool][], bigint]> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetPositions,
      [owner, size, offset]
    )
    if (result.ok) {
      const parsedResult = parse(result.ok) as [[Position, Pool][], bigint]
      let positionList = parsedResult[0]
      positionList = positionList.map(positionPoolPair => {
        return [
          deserializePositionFromContract(positionPoolPair[0]),
          deserializePoolFromContract(positionPoolPair[1])
        ]
      })

      return [positionList, parsedResult[1]]
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getAllPositions(
    owner: string,
    positionsCount?: bigint,
    skipPages?: number[],
    positionsPerPage?: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<Page[]> {
    const firstPageIndex = skipPages?.find(i => !skipPages.includes(i)) || 0
    const positionsPerPageLimit = positionsPerPage || POSITIONS_ENTRIES_LIMIT

    let pages: Page[] = []
    let actualPositionsCount = positionsCount
    if (!positionsCount) {
      const [positionEntries, positionsCount] = await this.getPositions(
        owner,
        positionsPerPageLimit,
        BigInt(firstPageIndex) * positionsPerPageLimit,
        options
      )

      pages.push({
        index: 0,
        entries: positionEntries
      })
      actualPositionsCount = positionsCount
    }

    const promises: Promise<[[Position, Pool][], bigint]>[] = []
    const pageIndexes: number[] = []

    for (
      let i = positionsCount ? firstPageIndex : firstPageIndex + 1;
      i < Math.ceil(Number(actualPositionsCount) / Number(positionsPerPageLimit));
      i++
    ) {
      if (skipPages?.includes(i)) {
        continue
      }

      pageIndexes.push(i)
      promises.push(
        this.getPositions(owner, positionsPerPageLimit, BigInt(i) * positionsPerPageLimit, options)
      )
    }

    const positionsEntriesList = await Promise.all(promises)
    pages = [
      ...pages,
      ...positionsEntriesList.map(([positionsEntries], index) => {
        return { index: pageIndexes[index], entries: positionsEntries }
      })
    ]

    return pages
  }

  createPositionTx(
    poolKey: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    liquidityDelta: Liquidity,
    spotSqrtPrice: SqrtPrice,
    slippageTolerance: Percentage,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    const slippageLimitLower = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      false
    )
    const slippageLimitUpper = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      true
    )

    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
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
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<CreatePositionTxResult> {
    const slippageLimitLower = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      false
    )
    const slippageLimitUpper = calculateSqrtPriceAfterSlippage(
      spotSqrtPrice,
      slippageTolerance,
      true
    )

    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.CreatePosition,
      [poolKey, lowerTick, upperTick, liquidityDelta, slippageLimitLower, slippageLimitUpper],
      this.waitForFinalization,
      block
    ) as Promise<CreatePositionTxResult>
  }

  transferPositionTx(
    index: bigint,
    receiver: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.TransferPosition,
      [index, receiver]
    )
  }

  async transferPosition(
    account: IKeyringPair,
    index: bigint,
    receiver: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.TransferPosition,
      [index, receiver],
      this.waitForFinalization,
      block
    )
  }

  removePositionTx(
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.RemovePosition,
      [index]
    )
  }

  async removePosition(
    account: IKeyringPair,
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<RemovePositionTxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.RemovePosition,
      [index],
      this.waitForFinalization,
      block
    ) as Promise<RemovePositionTxResult>
  }

  claimFeeTx(
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.ClaimFee,
      [index]
    )
  }

  async claimFee(
    account: IKeyringPair,
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.ClaimFee,
      [index],
      this.waitForFinalization,
      block
    )
  }

  async getTick(
    key: PoolKey,
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<Tick> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetTick,
      [key, index]
    )

    if (result.ok) {
      return deserializeTickFromContract(parse(result.ok))
    } else {
      throw new Error(extractError(result.err))
    }
  }

  async isTickInitialized(
    key: PoolKey,
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.IsTickInitialized,
      [key, index]
    )
  }

  async getPool(
    token0: string,
    token1: string,
    feeTier: FeeTier,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<Pool> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetPool,
      [token0, token1, feeTier]
    )

    if (result.ok) {
      return deserializePoolFromContract(parse(result.ok))
    } else {
      throw new Error(extractError(result.err))
    }
  }

  async getPoolKeys(
    size: bigint,
    offset: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<[PoolKey[], bigint]> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetPoolKeys,
      [size, offset]
    )
    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(extractError(result.err))
    }
  }

  async getAllPoolKeys(
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<PoolKey[]> {
    const [poolKeys, poolKeysCount] = await this.getPoolKeys(MAX_POOL_KEYS_RETURNED, 0n, options)

    const promises: Promise<[PoolKey[], bigint]>[] = []
    for (let i = 1; i < Math.ceil(Number(poolKeysCount) / Number(MAX_POOL_KEYS_RETURNED)); i++) {
      promises.push(
        this.getPoolKeys(MAX_POOL_KEYS_RETURNED, BigInt(i) * MAX_POOL_KEYS_RETURNED, options)
      )
    }

    const poolKeysEntries = await Promise.all(promises)
    return [...poolKeys, ...poolKeysEntries.map(([poolKeys]) => poolKeys).flat(1)]
  }

  createPoolTx(
    poolKey: PoolKey,
    initSqrtPrice: SqrtPrice,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    const initTick = calculateTick(initSqrtPrice, poolKey.feeTier.tickSpacing)

    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.CreatePool,
      [poolKey.tokenX, poolKey.tokenY, poolKey.feeTier, initSqrtPrice, initTick]
    )
  }

  async createPool(
    account: IKeyringPair,
    poolKey: PoolKey,
    initSqrtPrice: SqrtPrice,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<TxResult> {
    const initTick = calculateTick(initSqrtPrice, poolKey.feeTier.tickSpacing)

    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
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
    byAmountIn: boolean,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<QuoteResult> {
    const sqrtPriceLimit: SqrtPrice = xToY
      ? getMinSqrtPrice(poolKey.feeTier.tickSpacing)
      : getMaxSqrtPrice(poolKey.feeTier.tickSpacing)

    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.Quote,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(extractError(result.err))
    }
  }

  async quoteRoute(
    amountIn: TokenAmount,
    swaps: SwapHop[],
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<TokenAmount> {
    return sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.QuoteRoute,
      [amountIn, swaps]
    )
  }

  swapTx(
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.Swap,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit]
    )
  }

  async swap(
    account: IKeyringPair,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    sqrtPriceLimit: SqrtPrice,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<SwapTxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.Swap,
      [poolKey, xToY, amount, byAmountIn, sqrtPriceLimit],
      this.waitForFinalization,
      block
    ) as Promise<SwapTxResult>
  }

  swapWithSlippageTx(
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    estimatedSqrtPrice: SqrtPrice,
    slippage: Percentage,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    const sqrtPriceAfterSlippage = calculateSqrtPriceAfterSlippage(
      estimatedSqrtPrice,
      slippage,
      !xToY
    )

    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.Swap,
      [
        poolKey,
        xToY,
        amount,
        byAmountIn,
        xToY ? sqrtPriceAfterSlippage - 1n : sqrtPriceAfterSlippage + 1n
      ]
    )
  }

  async swapWithSlippage(
    account: IKeyringPair,
    poolKey: PoolKey,
    xToY: boolean,
    amount: TokenAmount,
    byAmountIn: boolean,
    estimatedSqrtPrice: SqrtPrice,
    slippage: Percentage,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<SwapTxResult> {
    const sqrtPriceAfterSlippage = calculateSqrtPriceAfterSlippage(
      estimatedSqrtPrice,
      slippage,
      !xToY
    )

    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.Swap,
      [
        poolKey,
        xToY,
        amount,
        byAmountIn,
        xToY ? sqrtPriceAfterSlippage - 1n : sqrtPriceAfterSlippage + 1n
      ],
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
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<SwapRouteTxResult> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.SwapRoute,
      [amountIn, expectedAmountOut, slippage, swaps],
      this.waitForFinalization,
      block
    ) as Promise<SwapRouteTxResult>
  }

  async getPositionWithAssociates(
    owner: string,
    index: bigint,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<[Position, Pool, Tick, Tick]> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetPositionWithAssociates,
      [owner, index]
    )

    if (result.ok) {
      const parsedResult = parse(result.ok)
      return [
        deserializePositionFromContract(parsedResult[0]),
        deserializePoolFromContract(parsedResult[1]),
        deserializeTickFromContract(parsedResult[2]),
        deserializeTickFromContract(parsedResult[3])
      ]
    } else {
      throw new Error(extractError(result.err))
    }
  }

  async getRawTickmap(
    poolKey: PoolKey,
    lowerTick: bigint,
    upperTick: bigint,
    xToY: boolean,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<[bigint, bigint][]> {
    return await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetTickmap,
      [poolKey, lowerTick, upperTick, xToY]
    )
  }

  async getFullTickmap(poolKey: PoolKey): Promise<Tickmap> {
    const maxTick = getMaxTick(poolKey.feeTier.tickSpacing)
    let lowerTick = getMinTick(poolKey.feeTier.tickSpacing)

    const xToY = false

    const promises: Promise<[bigint, bigint][]>[] = []
    const tickSpacing = poolKey.feeTier.tickSpacing
    assert(tickSpacing <= 100)

    assert(MAX_TICKMAP_QUERY_SIZE > 3)
    assert(CHUNK_SIZE * 2n > tickSpacing)
    // move back 1 chunk since the range is inclusive
    // then move back additional 2 chunks to ensure that adding tickspacing won't exceed the query limit
    const jump = (MAX_TICKMAP_QUERY_SIZE - 3n) * CHUNK_SIZE

    while (lowerTick <= maxTick) {
      let nextTick = lowerTick + jump
      const remainder = nextTick % tickSpacing

      if (remainder > 0) {
        nextTick += tickSpacing - remainder
      } else if (remainder < 0) {
        nextTick -= remainder
      }

      let upperTick = nextTick

      if (upperTick > maxTick) {
        upperTick = maxTick
      }

      assert(upperTick % tickSpacing === 0n)
      assert(lowerTick % tickSpacing === 0n)

      const result = this.getRawTickmap(poolKey, lowerTick, upperTick, xToY)
      promises.push(result)

      lowerTick = upperTick + tickSpacing
    }

    const fullResult: [bigint, bigint][] = (await Promise.all(promises)).flat(1)

    const storedTickmap = new Map<bigint, bigint>(fullResult)

    return { bitmap: storedTickmap }
  }

  async getLiquidityTicks(
    poolKey: PoolKey,
    ticks: bigint[],
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<LiquidityTick[]> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetLiquidityTicks,
      [poolKey, ticks]
    )

    if (result.ok) {
      return parse(result.ok)
    } else {
      throw new Error(InvariantError[result.err])
    }
  }

  async getAllLiquidityTicks(poolKey: PoolKey, tickmap: Tickmap): Promise<LiquidityTick[]> {
    const tickIndexes: bigint[] = []
    for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
      for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
        const checkedBit = chunk & (1n << bit)
        if (checkedBit) {
          const tickIndex = positionToTick(chunkIndex, bit, poolKey.feeTier.tickSpacing)
          tickIndexes.push(tickIndex)
        }
      }
    }
    const tickLimit = integerSafeCast(LIQUIDITY_TICKS_LIMIT)
    const promises: Promise<LiquidityTick[]>[] = []
    for (let i = 0; i < tickIndexes.length; i += tickLimit) {
      promises.push(this.getLiquidityTicks(poolKey, tickIndexes.slice(i, i + tickLimit)))
    }

    const tickResults = await Promise.all(promises)
    return tickResults.flat(1)
  }

  async getUserPositionAmount(
    owner: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<bigint> {
    return sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetUserPositionAmount,
      [owner]
    )
  }

  withdrawAllWAZEROTx(
    address: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.WithdrawAllWAZERO,
      [address]
    )
  }

  async withdrawAllWAZERO(
    account: IKeyringPair,
    address: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<any> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.WithdrawAllWAZERO,
      [address],
      this.waitForFinalization,
      block
    )
  }

  async getAllPoolsForPair(
    token0: string,
    token1: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): Promise<[FeeTier, Pool][]> {
    const result = await sendQuery(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      InvariantQuery.GetAllPoolsForPair,
      [token0, token1]
    )

    if (result.ok) {
      return (parse(result.ok) as any[]).map(value => [
        value[0],
        deserializePoolFromContract(value[1])
      ])
    } else {
      throw new Error(result.err ? InvariantError[result.err] : result)
    }
  }

  setCodeTx(
    codeHash: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    }
  ): SubmittableExtrinsic<'promise'> {
    return createTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      InvariantTx.SetCode,
      [codeHash]
    )
  }

  async setCode(
    account: IKeyringPair,
    codeHash: string,
    options: ContractOptions = {
      storageDepositLimit: this.storageDepositLimit,
      refTime: this.gasLimit.refTime.toNumber(),
      proofSize: this.gasLimit.proofSize.toNumber()
    },
    block: boolean = true
  ): Promise<any> {
    return createSignAndSendTx(
      this.contract,
      this.api.registry.createType('WeightV2', {
        refTime: options.refTime,
        proofSize: options.proofSize
      }) as WeightV2,
      options.storageDepositLimit,
      0n,
      account,
      InvariantTx.SetCode,
      [codeHash],
      this.waitForFinalization,
      block
    )
  }
}
