import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { Bytes } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { PSP22Query, PSP22Tx } from './schema.js'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME, sendQuery, sendTx } from './utils.js'

export class PSP22 {
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
    supply: bigint,
    name: string,
    symbol: string,
    decimals: number
  ): Promise<DeployedContract> {
    return deployContract(this.api, account, abi, wasm, 'new', [supply, name, symbol, decimals])
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  async mint(account: IKeyringPair, value: number, block: boolean = true): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      PSP22Tx.Mint,
      [value],
      this.waitForFinalization,
      block
    )
  }

  async transfer(
    account: IKeyringPair,
    to: string,
    value: number,
    data: Bytes,
    block: boolean = true
  ): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      PSP22Tx.Transfer,
      [to, value, data],
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

  async tokenName(account: IKeyringPair): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TokenName,
      []
    )
  }

  async tokenSymbol(account: IKeyringPair): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TokenSymbol,
      []
    )
  }

  async tokenDecimals(account: IKeyringPair): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TokenDecimals,
      []
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

  async totalSupply(account: IKeyringPair): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TotalSupply,
      []
    )
  }

  async allowance(account: IKeyringPair, owner: string, spender: string): Promise<unknown> {
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.Allowance,
      [owner, spender]
    )
  }
}
