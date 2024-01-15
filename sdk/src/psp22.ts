import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { Bytes } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { PSP22Query, PSP22Tx, TxResult } from './schema.js'
import {
  DEFAULT_PROOF_SIZE,
  DEFAULT_REF_TIME,
  getDeploymentData,
  sendQuery,
  sendTx
} from './utils.js'

export class PSP22 {
  contract: ContractPromise
  api: ApiPromise
  gasLimit: WeightV2
  storageDepositLimit: number | null = null
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
    this.waitForFinalization = network !== Network.Local
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  static async getContract(
    api: ApiPromise,
    network: Network,
    storageDepositLimit: number | null = null,
    refTime: number = DEFAULT_REF_TIME,
    proofSize: number = DEFAULT_PROOF_SIZE,
    account: IKeyringPair,
    supply: bigint,
    name: string,
    symbol: string,
    decimals: bigint,
    address?: string
  ): Promise<PSP22> {
    const tokenData = await getDeploymentData('psp22')
    if (address && network != Network.Local) {
      return new PSP22(
        api,
        network,
        storageDepositLimit,
        refTime,
        proofSize,
        tokenData.abi,
        address
      )
    } else {
      const tokenDeploy = await PSP22.deploy(
        api,
        account,
        tokenData.abi,
        tokenData.wasm,
        supply,
        name,
        symbol,
        decimals
      )

      return new PSP22(
        api,
        Network.Local,
        storageDepositLimit,
        refTime,
        proofSize,
        tokenData.abi,
        tokenDeploy.address
      )
    }
  }

  static async deploy(
    api: ApiPromise,
    account: IKeyringPair,
    abi: any,
    wasm: Buffer,
    supply: bigint,
    name: string,
    symbol: string,
    decimals: bigint
  ): Promise<DeployedContract> {
    return deployContract(api, account, abi, wasm, 'new', [supply, name, symbol, decimals])
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

  async balanceOf(account: IKeyringPair, owner: string): Promise<bigint> {
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
