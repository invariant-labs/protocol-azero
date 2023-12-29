import { ApiPromise } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { DeployedContract } from '@scio-labs/use-inkathon'
import { deployContract } from '@scio-labs/use-inkathon/helpers'
import { Network } from './network.js'
import { InvariantQuery, InvariantTx } from './schema.js'

export class Invariant {
  public static readonly DEFAULT_REF_TIME = 100000000000
  public static readonly DEFAULT_PROOF_SIZE = 100000000000

  contract: ContractPromise | null = null
  api: ApiPromise
  account: IKeyringPair
  weight: WeightV2
  waitForFinalization: boolean

  constructor(
    api: ApiPromise,
    account: IKeyringPair,
    network: Network,
    refTime: number = Invariant.DEFAULT_REF_TIME,
    proofSize: number = Invariant.DEFAULT_PROOF_SIZE
  ) {
    this.api = api
    this.account = account
    this.weight = api.registry.createType('WeightV2', {
      refTime,
      proofSize
    }) as WeightV2
    this.waitForFinalization = network != Network.Local
  }

  async load(deploymentAddress: string, abi: any): Promise<void> {
    this.contract = new ContractPromise(this.api, abi, deploymentAddress)
  }

  async deploy(abi: any, wasm: Buffer, fee: { v: number }): Promise<DeployedContract> {
    return await deployContract(this.api, this.account, abi, wasm, 'new', [fee])
  }

  async sendQuery(message: InvariantQuery, signer: string, params: any[]): Promise<void> {
    if (!this.contract) {
      throw new Error('contract not loaded')
    }

    const { result, output } = await this.contract.query[message](
      signer,
      {
        gasLimit: this.weight,
        storageDepositLimit: null
      },
      ...params
    )

    if (result.isOk && output) {
      return JSON.parse(output.toString()).ok
    } else {
      throw new Error(result.asErr.toHuman()?.toString())
    }
  }

  async sendTx(
    message: InvariantTx,
    signer: IKeyringPair,
    params: any[],
    block: boolean = true
  ): Promise<string> {
    if (!this.contract) {
      throw new Error('contract not loaded')
    }

    const call = this.contract.tx[message](
      {
        gasLimit: this.weight,
        storageDepositLimit: null
      },
      ...params
    )

    return new Promise<string>(async (resolve, reject) => {
      await call.signAndSend(signer, result => {
        if (!block) {
          resolve(result.txHash.toHex())
        }
        if (result.isError) {
          reject(result.toHuman())
        }
        if (result.isCompleted && !this.waitForFinalization) {
          resolve(result.txHash.toHex())
        }
        if (result.isFinalized) {
          resolve(result.txHash.toHex())
        }
      })
    })
  }

  async getProtocolFee(): Promise<void> {
    return await this.sendQuery(InvariantQuery.ProtocolFee, this.account.address, [])
  }

  async changeProtocolFee(fee: { v: bigint }): Promise<string> {
    return await this.sendTx(InvariantTx.ChangeProtocolFee, this.account, [fee])
  }
}
