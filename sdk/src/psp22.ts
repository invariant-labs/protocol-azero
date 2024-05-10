import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { Bytes } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { deployContract } from '@scio-labs/use-inkathon'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME } from './consts.js'
import { Network } from './network.js'
import { ContractOptions, PSP22Query, PSP22Tx, TxResult } from './schema.js'
import { getAbi, getDeploymentData, getTx, sendQuery, sendTx } from './utils.js'

export class PSP22 {
  contract: ContractPromise
  api: ApiPromise
  abi: any
  gasLimit: WeightV2
  storageDepositLimit: number | null = null
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
    this.abi = abi
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
    deployer: IKeyringPair,
    supply: bigint = 0n,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 0n
  ): Promise<string> {
    const deploymentData = await getDeploymentData('psp22')
    const deploy = await deployContract(
      api,
      deployer,
      deploymentData.abi,
      deploymentData.wasm,
      'new',
      [supply, name, symbol, decimals]
    )

    return deploy.address.toString()
  }

  static async load(
    api: ApiPromise,
    network: Network,
    address: string,
    options?: ContractOptions
  ): Promise<PSP22> {
    const abi = await getAbi('psp22')

    return new PSP22(
      api,
      network,
      abi,
      address,
      options?.storageDepositLimit,
      options?.refTime,
      options?.proofSize
    )
  }

  async setContractAddress(address: string) {
    this.contract = new ContractPromise(this.api, this.abi, address)
  }

  mintTx(value: bigint) {
    return getTx(this.contract, this.gasLimit, this.storageDepositLimit, 0n, PSP22Tx.Mint, [value])
  }

  async mint(account: IKeyringPair, value: bigint, block: boolean = true): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      PSP22Tx.Mint,
      [value],
      this.waitForFinalization,
      block
    )
  }

  transferTx(to: string, value: bigint, data: Bytes) {
    return getTx(this.contract, this.gasLimit, this.storageDepositLimit, 0n, PSP22Tx.Transfer, [
      to,
      value,
      data
    ])
  }

  async transfer(
    account: IKeyringPair,
    to: string,
    value: bigint,
    data: Bytes,
    block: boolean = true
  ): Promise<TxResult> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0n,
      account,
      PSP22Tx.Transfer,
      [to, value, data],
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

  async tokenName(userAddress: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      userAddress,
      PSP22Query.TokenName,
      []
    )
  }

  async tokenSymbol(userAddress: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      userAddress,
      PSP22Query.TokenSymbol,
      []
    )
  }

  async tokenDecimals(userAddress: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      userAddress,
      PSP22Query.TokenDecimals,
      []
    )
  }

  async balanceOf(userAddress: string, owner: string): Promise<bigint> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      userAddress,
      PSP22Query.BalanceOf,
      [owner]
    )
  }

  async totalSupply(userAddress: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      userAddress,
      PSP22Query.TotalSupply,
      []
    )
  }

  async allowance(userAddress: string, owner: string, spender: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      userAddress,
      PSP22Query.Allowance,
      [owner, spender]
    )
  }
}
