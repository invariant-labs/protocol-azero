import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { Codec } from '@polkadot/types/types'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { FeeTier, InvariantQuery, InvariantTx, PoolKey } from './schema.js'
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
    fee: { v: bigint }
  ): Promise<DeployedContract> {
    return deployContract(this.api, account, abi, wasm, 'new', [fee])
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  async getProtocolFee(account: IKeyringPair): Promise<{ v: bigint }> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.ProtocolFee,
      []
    ) as Promise<{ v: bigint }>
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

  async addFeeTier(
    account: IKeyringPair,
    fee_tier: FeeTier,
    block: boolean = true
  ): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.AddFeeTier,
      [fee_tier],
      this.waitForFinalization,
      block
    )
  }

  async removeFeeTier(
    account: IKeyringPair,
    fee_tier: FeeTier,
    block: boolean = true
  ): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.RemoveFeeTier,
      [fee_tier],
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

  async feeTierExist(account: IKeyringPair, fee_tier: FeeTier): Promise<boolean> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      InvariantQuery.FeeTierExist,
      [fee_tier]
    ) as Promise<boolean>
  }

  // TODO: test this function
  async changeFeeReceiver(
    account: IKeyringPair,
    pool_key: PoolKey,
    fee_receiver: Codec,
    block: boolean = true
  ): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.ChangeFeeReceiver,
      [pool_key, fee_receiver],
      this.waitForFinalization,
      block
    )
  }

  // TODO: test this function
  async withdrawProtocolFee(
    account: IKeyringPair,
    pool_key: PoolKey,
    block: boolean = true
  ): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      InvariantTx.WithdrawProtocolFee,
      [pool_key],
      this.waitForFinalization,
      block
    )
  }
}
