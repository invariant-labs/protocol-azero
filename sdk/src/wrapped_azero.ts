import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { PSP22Query, PSP22Tx, WrappedAZEROTx } from './schema.js'
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

  static async create(api: ApiPromise, account: IKeyringPair) {
    const tokenData = await getDeploymentData('wrapped_azero')

    const tokenDeploy = await deployContract(api, account, tokenData.abi, tokenData.wasm, 'new', [])

    return new WrappedAZERO(
      api,
      Network.Local,
      null,
      DEFAULT_REF_TIME,
      DEFAULT_PROOF_SIZE,
      tokenData.abi,
      tokenDeploy.address
    )
  }

  async deploy(account: IKeyringPair, abi: any, wasm: Buffer): Promise<DeployedContract> {
    return deployContract(this.api, account, abi, wasm, 'new', [])
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  async deposit(account: IKeyringPair, value: number, block: boolean = true): Promise<string> {
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

  async withdraw(account: IKeyringPair, value: number, block: boolean = true): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
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
    value: number,
    block: boolean = true
  ): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
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
