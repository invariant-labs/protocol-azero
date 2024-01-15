import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { PSP22Query, PSP22Tx, TxResult, WrappedAZEROTx } from './schema.js'
import {
  DEFAULT_PROOF_SIZE,
  DEFAULT_REF_TIME,
  getDeploymentData,
  sendQuery,
  sendTx
} from './utils.js'

export class WrappedAZERO {
  contract: ContractPromise
  api: ApiPromise
  gasLimit: WeightV2
  storageDepositLimit: number | null
  waitForFinalization: boolean

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
  }

  static async getContract(
    api: ApiPromise,
    account: IKeyringPair,
    storageDepositLimit: number | null = null,
    refTime: number = DEFAULT_REF_TIME,
    proofSize: number = DEFAULT_PROOF_SIZE,
    network: Network
  ): Promise<WrappedAZERO> {
    const wazeroData = await getDeploymentData('wrapped_azero')

    if (process.env.WAZERO_ADDRESS && network !== Network.Local) {
      return new WrappedAZERO(
        api,
        network,
        storageDepositLimit,
        refTime,
        proofSize,
        wazeroData.abi,
        process.env.WAZERO_ADDRESS
      )
    } else {
      const wazeroDeploy = await WrappedAZERO.deploy(api, account, wazeroData.abi, wazeroData.wasm)
      return new WrappedAZERO(
        api,
        network,
        storageDepositLimit,
        refTime,
        proofSize,
        wazeroData.abi,
        wazeroDeploy.address
      )
    }
  }

  static async deploy(
    api: ApiPromise,
    account: IKeyringPair,
    abi: any,
    wasm: Buffer
  ): Promise<DeployedContract> {
    return deployContract(api, account, abi, wasm, 'new', [])
  }

  async deposit(account: IKeyringPair, value: bigint, block: boolean = true): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      value,
      account,
      WrappedAZEROTx.Deposit,
      [],
      this.waitForFinalization,
      block
    )
  }

  async withdraw(account: IKeyringPair, value: bigint, block: boolean = true): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      WrappedAZEROTx.Withdraw,
      [value],
      this.waitForFinalization,
      block
    )
  }

  async approve(
    account: IKeyringPair,
    spender: string,
    value: bigint,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      PSP22Tx.Approve,
      [spender, value],
      this.waitForFinalization,
      block
    )
  }

  async balanceOf(account: IKeyringPair, owner: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.BalanceOf,
      [owner]
    )
  }
}
