import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { Bytes } from '@polkadot/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { ContractOptions, PSP22Query, PSP22Tx, TxResult } from './schema.js'
import {
  DEFAULT_PROOF_SIZE,
  DEFAULT_REF_TIME,
  getDeploymentData,
  isLoaded,
  sendQuery,
  sendTx
} from './utils.js'

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
    network: Network,
    deployer: IKeyringPair,
    supply: bigint = 0n,
    name: string = '',
    symbol: string = '',
    decimals: bigint = 0n,
    options?: ContractOptions
  ): Promise<PSP22> {
    const deploymentData = await getDeploymentData('psp22')
    const deploy = await deployContract(
      api,
      deployer,
      deploymentData.abi,
      deploymentData.wasm,
      'new',
      [supply, name, symbol, decimals]
    )

    return new PSP22(
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
    address: string,
    options?: ContractOptions
  ): Promise<PSP22> {
    const deploymentData = await getDeploymentData('psp22')

    return new PSP22(
      api,
      network,
      deploymentData.abi,
      address,
      options?.storageDepositLimit,
      options?.refTime,
      options?.proofSize
    )
  }

  async load(address: string) {
    this.contract = new ContractPromise(this.api, this.abi, address)
  }

  async mint(
    account: IKeyringPair,
    value: bigint,
    address: string = this.contract.address.toString(),
    block: boolean = true
  ): Promise<TxResult> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
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
    address: string = this.contract.address.toString(),
    block: boolean = true
  ): Promise<TxResult> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
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
    address: string = this.contract.address.toString(),
    block: boolean = true
  ): Promise<TxResult> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
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

  async tokenName(
    account: IKeyringPair,
    address: string = this.contract.address.toString()
  ): Promise<unknown> {
    if (!isLoaded(address, this.contract.address.toString())) {
      await this.load(address)
    }
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TokenName,
      []
    )
  }

  async tokenSymbol(
    account: IKeyringPair,
    address: string = this.contract.address.toString()
  ): Promise<unknown> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TokenSymbol,
      []
    )
  }

  async tokenDecimals(
    account: IKeyringPair,
    address: string = this.contract.address.toString()
  ): Promise<unknown> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TokenDecimals,
      []
    )
  }

  async balanceOf(
    account: IKeyringPair,
    owner: string,
    address: string = this.contract.address.toString()
  ): Promise<bigint> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.BalanceOf,
      [owner]
    )
  }

  async totalSupply(
    account: IKeyringPair,
    address: string = this.contract.address.toString()
  ): Promise<unknown> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
    return sendQuery(
      this.contract,
      this.gasLimit,
      this.storageDepositLimit,
      account,
      PSP22Query.TotalSupply,
      []
    )
  }

  async allowance(
    account: IKeyringPair,
    owner: string,
    spender: string,
    address: string = this.contract.address.toString()
  ): Promise<unknown> {
    if (!isLoaded(address, this.contract.address.toString())) {
      this.load(address)
    }
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
