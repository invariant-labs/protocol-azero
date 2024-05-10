import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { deployContract } from '@scio-labs/use-inkathon'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME } from './consts.js'
import { Network } from './network.js'
import { ContractOptions, PSP22Query, PSP22Tx, TxResult, WrappedAZEROTx } from './schema.js'
import { getAbi, getDeploymentData, getTx, sendQuery, sendTx } from './utils.js'

export class WrappedAZERO {
  contract: ContractPromise
  api: ApiPromise
  gasLimit: WeightV2
  storageDepositLimit: number | null
  waitForFinalization: boolean

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
  }

  static async deploy(
    api: ApiPromise,
    network: Network,
    deployer: IKeyringPair,
    options?: ContractOptions
  ): Promise<WrappedAZERO> {
    const deploymentData = await getDeploymentData('wrapped-azero')
    const deploy = await deployContract(
      api,
      deployer,
      deploymentData.abi,
      deploymentData.wasm,
      'new',
      []
    )

    return new WrappedAZERO(
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
    deployer: string,
    options?: ContractOptions
  ): Promise<WrappedAZERO> {
    const abi = await getAbi('wrapped-azero')

    return new WrappedAZERO(
      api,
      network,
      abi,
      deployer,
      options?.storageDepositLimit,
      options?.refTime,
      options?.proofSize
    )
  }

  // approveTx(spender: string, value: bigint) {
  //   return getTx(this.contract, this.gasLimit, this.storageDepositLimit, 0n, PSP22Tx.Approve, [
  //     spender,
  //     value
  //   ])
  // }

  depositTx(value: bigint) {
    return getTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      value,
      WrappedAZEROTx.Deposit,
      []
    )
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

  withdrawTx(value: bigint) {
    return getTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      WrappedAZEROTx.Withdraw,
      [value]
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

  approveTx(spender: string, value: bigint) {
    return getTx(this.contract, this.gasLimit, this.storageDepositLimit, 0n, PSP22Tx.Approve, [
      spender,
      value
    ])
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

  async balanceOf(userAddress: string, owner: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      userAddress,
      PSP22Query.BalanceOf,
      [owner]
    )
  }
}
