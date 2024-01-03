import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { FeeTier, Liquidity, Pool, PoolKey, Position, SqrtPrice, Tick } from 'math'
import { Network } from './network.js'
import { InvariantQuery, InvariantTx } from './schema.js'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME, sendQuery, sendTx } from './utils.js'

export class Invariant {
  contract: ContractPromise | null = null
  api: ApiPromise
  gasLimit: WeightV2
  storageDepositLimit: number | null
  waitForFinalization: boolean

  constructor(
    api: ApiPromise,
    network: Network,
    storageDepositLimit: number | null = null,
    refTime: number = DEFAULT_REF_TIME,
    proofSize: number = DEFAULT_PROOF_SIZE
  ) {
    this.api = api
    this.gasLimit = api.registry.createType('WeightV2', {
      refTime,
      proofSize
    }) as WeightV2
    this.storageDepositLimit = storageDepositLimit
    this.waitForFinalization = network != Network.Local
  }

  async deploy(
    account: IKeyringPair,
    abi: any,
    wasm: Buffer,
    fee: { v: number }
  ): Promise<DeployedContract> {
    return deployContract(this.api, account, abi, wasm, 'new', [fee])
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  async getProtocolFee(account: IKeyringPair): Promise<unknown> {
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
    fee: { v: bigint },
    block: boolean = true
  ): Promise<string> {
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

  async getPool(
    account: IKeyringPair,
    token0: string,
    token1: string,
    fee_tier: FeeTier
  ): Promise<Pool> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPool,
      [token0, token1, fee_tier]
    ) as Promise<Pool>
  }

  async getPools(account: IKeyringPair): Promise<Pool[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPools,
      []
    ) as Promise<Pool[]>
  }

  async createPool(
    account: IKeyringPair,
    token_0: string,
    token_1: string,
    feeTier: FeeTier,
    initSqrtPrice: SqrtPrice,
    initTick: bigint,
    block: boolean = true
  ): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.CreatePool,
      [token_0, token_1, feeTier, initSqrtPrice, initTick],
      this.waitForFinalization,
      block
    )
  }

  async getPosition(account: IKeyringPair, index: bigint): Promise<Position> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPosition,
      [index]
    ) as Promise<Position>
  }

  async getPositions(account: IKeyringPair): Promise<Position[]> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetAllPositions,
      []
    ) as Promise<Position[]>
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
  ): Promise<string> {
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
  ): Promise<string> {
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
  ): Promise<string> {
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

  async claimFee(account: IKeyringPair, index: bigint, block: boolean = true): Promise<string> {
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

  async isTickInitialized(account: IKeyringPair, key: PoolKey, index: bigint): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.IsTickInitialized,
      [key, index]
    ) as Promise<boolean>
  }

  async getTick(account: IKeyringPair, key: PoolKey, index: bigint): Promise<Tick> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetTick,
      [key, index]
    ) as Promise<Tick>
  }
}
