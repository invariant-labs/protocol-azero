import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { PSP22Query, PSP22Tx, WrappedAZEROTx } from './schema.js'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME, sendQuery, sendTx } from './utils.js'

export class WrappedAZERO {
  contract: ContractPromise | null = null
  api: ApiPromise
  gasLimit: WeightV2
  storageDepositLimit: number | null

  constructor(
    api: ApiPromise,
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
  }

  async deploy(account: IKeyringPair, abi: any, wasm: Buffer): Promise<DeployedContract> {
    return deployContract(this.api, account, abi, wasm, 'new', [])
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  async deposit(account: IKeyringPair, value: number): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      value,
      account,
      WrappedAZEROTx.Deposit,
      []
    )
  }

  async depositWithoutFinalization(account: IKeyringPair, value: number): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      value,
      account,
      WrappedAZEROTx.Deposit,
      [],
      false
    )
  }

  async withdraw(account: IKeyringPair, value: number): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      WrappedAZEROTx.Withdraw,
      [value]
    )
  }

  async withdrawWithoutFinalization(account: IKeyringPair, value: number): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      WrappedAZEROTx.Withdraw,
      [value],
      false
    )
  }

  async approve(account: IKeyringPair, spender: string, value: number): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      PSP22Tx.Approve,
      [spender, value]
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
