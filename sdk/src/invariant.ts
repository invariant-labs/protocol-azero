import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { FeeTier, Liquidity, Percentage, Pool, PoolKey, Position, SqrtPrice, Tick } from 'math'
import { Network } from './network.js'
import { InvariantQuery, InvariantTx, Result } from './schema.js'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME, convertObj, sendQuery, sendTx } from './utils.js'

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
    fee: Percentage
  ): Promise<DeployedContract> {
    return deployContract(this.api, account, abi, wasm, 'new', [fee])
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
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

  async addFeeTier(
    account: IKeyringPair,
    feeTier: FeeTier,
    block: boolean = true
  ): Promise<string> {
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
  ): Promise<string> {
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
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetFeeTiers,
      []
    ) as Promise<FeeTier[]>
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
  ): Promise<string> {
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
  ): Promise<string> {
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

  async getTick(account: IKeyringPair, key: PoolKey, index: bigint): Promise<Result<Tick>> {
    const result = (await sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetTick,
      [key, index]
    )) as any

    if (result.ok) {
      return {
        ok: convertObj(result.ok)
      }
    } else {
      return {
        err: result.err
      }
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
    ) as Promise<boolean>
  }

  async getPool(
    account: IKeyringPair,
    token0: string,
    token1: string,
    feeTier: FeeTier
  ): Promise<Pool> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.GetPool,
      [token0, token1, feeTier]
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
    token0: string,
    token1: string,
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
      [token0, token1, feeTier, initSqrtPrice, initTick],
      this.waitForFinalization,
      block
    )
  }
}
