import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { Bytes, Option } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { Codec, IKeyringPair } from '@polkadot/types/types'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { PSP22Query, PSP22Tx } from './schema.js'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME, sendQuery, sendTx } from './utils.js'

export enum PSP22Message {
  Mint = 'psp22Mintable::mint',
  Transfer = 'psp22::transfer',
  Approve = 'psp22::approve',
  TokenName = 'psp22Metadata::tokenName',
  TokenSymbol = 'psp22Metadata::tokenSymbol',
  TokenDecimals = 'psp22Metadata::tokenDecimals',
  BalanceOf = 'psp22::balanceOf',
  TotalSupply = 'psp22::totalSupply',
  Allowance = 'psp22::allowance'
}

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
    supply: number,
    name: Option<Codec>,
    symbol: Option<Codec>,
    decimals: number
  ): Promise<DeployedContract> {
    return deployContract(this.api, account, abi, wasm, 'new', [supply, name, symbol, decimals])
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  async mint(account: IKeyringPair, value: number): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      PSP22Tx.Mint,
      [value],
      this.waitForFinalization
    )
  }

  async mintWithoutFinalization(account: IKeyringPair, value: number): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      PSP22Tx.Mint,
      [value],
      this.waitForFinalization,
      false
    )
  }

  async transfer(account: IKeyringPair, to: string, value: number, data: Bytes): Promise<string> {
    return sendTx(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      0,
      account,
      PSP22Tx.Transfer,
      [to, value, data],
      this.waitForFinalization
    )
  }

  async transferWithoutFinalization(
    account: IKeyringPair,
    to: string,
    value: number,
    data: Bytes
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
