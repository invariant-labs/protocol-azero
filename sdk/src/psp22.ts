import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { SubmittableExtrinsic } from '@polkadot/api/promise/types'
import { Bytes } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { deployContract } from '@scio-labs/use-inkathon'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME } from './consts.js'
import { Network } from './network.js'
import { ContractOptions, PSP22Query, PSP22Tx, TxResult } from './schema.js'
import { createSignAndSendTx, createTx, getAbi, getDeploymentData, sendQuery } from './utils.js'

export class PSP22 {
  api: ApiPromise
  abi: any
  gasLimit: WeightV2
  storageDepositLimit: number | null = null
  waitForFinalization: boolean

  private constructor(
    api: ApiPromise,
    network: Network,
    abi: any,
    storageDepositLimit: number | null = null,
    refTime: number = DEFAULT_REF_TIME,
    proofSize: number = DEFAULT_PROOF_SIZE
  ) {
    this.api = api
    this.abi = abi
    this.waitForFinalization = network !== Network.Local
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

  static async load(api: ApiPromise, network: Network, options?: ContractOptions): Promise<PSP22> {
    const abi = await getAbi('psp22')

    return new PSP22(
      api,
      network,
      abi,
      options?.storageDepositLimit,
      options?.refTime,
      options?.proofSize
    )
  }

  mintTx(value: bigint, tokenAddress: string): SubmittableExtrinsic {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return createTx(contract, this.gasLimit, this.storageDepositLimit, 0n, PSP22Tx.Mint, [value])
  }

  async mint(
    account: IKeyringPair,
    value: bigint,
    tokenAddress: string,
    block: boolean = true
  ): Promise<TxResult> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return createSignAndSendTx(
      contract,
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

  transferTx(to: string, value: bigint, data: Bytes, tokenAddress: string): SubmittableExtrinsic {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return createTx(contract, this.gasLimit, this.storageDepositLimit, 0n, PSP22Tx.Transfer, [
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
    tokenAddress: string,
    block: boolean = true
  ): Promise<TxResult> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return createSignAndSendTx(
      contract,
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

  approveTx(spender: string, value: bigint, tokenAddress: string): SubmittableExtrinsic {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return createTx(contract, this.gasLimit, this.storageDepositLimit, 0n, PSP22Tx.Approve, [
      spender,
      value
    ])
  }

  async approve(
    account: IKeyringPair,
    spender: string,
    value: bigint,
    tokenAddress: string,
    block: boolean = true
  ): Promise<TxResult> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return createSignAndSendTx(
      contract,
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

  async tokenName(tokenAddress: string): Promise<unknown> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return sendQuery(contract, this.gasLimit, this.storageDepositLimit, PSP22Query.TokenName, [])
  }

  async tokenSymbol(tokenAddress: string): Promise<unknown> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return sendQuery(contract, this.gasLimit, this.storageDepositLimit, PSP22Query.TokenSymbol, [])
  }

  async tokenDecimals(tokenAddress: string): Promise<unknown> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return sendQuery(
      contract,
      this.gasLimit,
      this.storageDepositLimit,
      PSP22Query.TokenDecimals,
      []
    )
  }

  async balanceOf(owner: string, tokenAddress: string): Promise<bigint> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return sendQuery(contract, this.gasLimit, this.storageDepositLimit, PSP22Query.BalanceOf, [
      owner
    ])
  }

  async totalSupply(tokenAddress: string): Promise<unknown> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return sendQuery(contract, this.gasLimit, this.storageDepositLimit, PSP22Query.TotalSupply, [])
  }

  async allowance(owner: string, spender: string, tokenAddress: string): Promise<unknown> {
    const contract = new ContractPromise(this.api, this.abi, tokenAddress)

    return sendQuery(contract, this.gasLimit, this.storageDepositLimit, PSP22Query.Allowance, [
      owner,
      spender
    ])
  }

  async getAllBalances(tokens: string[], owner: string): Promise<Map<string, bigint>> {
    const balancePromises = await Promise.all(tokens.map(token => this.balanceOf(owner, token)));
    
    return new Map(tokens.map((token, i) => [token, balancePromises[i]]));
  }
}
