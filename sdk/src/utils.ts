import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { getSubstrateChain } from '@scio-labs/use-inkathon/chains'
import { getBalance, initPolkadotJs as initApi } from '@scio-labs/use-inkathon/helpers'
import { readFile } from 'fs/promises'
import { InvariantError, Percentage } from 'math'
import { Invariant } from './invariant.js'
import { Network } from './network.js'
import { PSP22 } from './psp22.js'
import { InvariantTx, Query, Tx } from './schema.js'
import { WrappedAZERO } from './wrapped_azero.js'

export const DEFAULT_REF_TIME = 100000000000
export const DEFAULT_PROOF_SIZE = 100000000000

export const initPolkadotApi = async (network: Network): Promise<ApiPromise> => {
  if (network === Network.Local) {
    const wsProvider = new WsProvider(process.env.LOCAL)
    const api = await ApiPromise.create({ provider: wsProvider })
    await api.isReady
    return api
  } else if (network === Network.Testnet) {
    const chainId = process.env.CHAIN
    const chain = getSubstrateChain(chainId)
    if (!chain) {
      throw new Error('chain not found')
    }
    const { api } = await initApi(chain, { noInitWarn: true })
    return api
  } else {
    throw new Error('Invalid network')
  }
}

export const getEnvAccount = async (keyring: Keyring): Promise<IKeyringPair> => {
  const accountUri = process.env.ACCOUNT_URI

  if (!accountUri) {
    throw new Error('invalid account uri')
  }

  return keyring.addFromUri(accountUri)
}

export const getEnvTestAccount = async (keyring: Keyring): Promise<IKeyringPair> => {
  const accountUri = process.env.TEST_ACCOUNT_URI

  if (!accountUri) {
    throw new Error('invalid account uri')
  }

  return keyring.addFromUri(accountUri)
}

export const printBalance = async (api: ApiPromise, account: IKeyringPair) => {
  const network = (await api.rpc.system.chain())?.toString() || ''
  const version = (await api.rpc.system.version())?.toString() || ''
  const balance = await getBalance(api, account.address)

  console.log(`network: ${network} (${version})`)
  console.log(`account: ${account.address} (${balance.balanceFormatted})\n`)
}

export const getDeploymentData = async (
  contractName: string
): Promise<{ abi: any; wasm: Buffer }> => {
  try {
    const abi = JSON.parse(
      await readFile(`./contracts/${contractName}/${contractName}.json`, 'utf-8')
    )
    const wasm = await readFile(`./contracts/${contractName}/${contractName}.wasm`)

    return { abi, wasm }
  } catch (error) {
    throw new Error(`${contractName}.json or ${contractName}.wasm not found`)
  }
}

export const sleep = async (ms: number) => {
  return await new Promise(resolve => setTimeout(resolve, ms))
}

export async function sendQuery(
  contract: ContractPromise | null,
  gasLimit: WeightV2,
  storageDepositLimit: number | null,
  signer: IKeyringPair,
  message: Query | Tx,
  data: any[]
): Promise<unknown> {
  if (!contract) {
    throw new Error('contract not loaded')
  }

  const { result, output } = await contract.query[message](
    signer.address,
    {
      gasLimit: gasLimit,
      storageDepositLimit: storageDepositLimit
    },
    ...data
  )

  if (result.isOk && output) {
    return JSON.parse(output.toString()).ok
  } else {
    throw new Error(result.asErr.toHuman()?.toString())
  }
}

export async function sendTx(
  contract: ContractPromise | null,
  gasLimit: WeightV2,
  storageDepositLimit: number | null,
  value: number,
  signer: IKeyringPair,
  message: Tx,
  data: any[],
  waitForFinalization: boolean = true,
  block: boolean = true
): Promise<string> {
  if (!contract) {
    throw new Error('contract not loaded')
  }

  const call = contract.tx[message](
    {
      gasLimit,
      storageDepositLimit,
      value
    },
    ...data
  )

  return new Promise<string>(async (resolve, reject) => {
    await call.signAndSend(signer, result => {
      if (!block) {
        resolve(result.txHash.toHex())
      }
      if (result.isError || result.dispatchError) {
        reject(new Error(message))
      }
      if (result.isCompleted && !waitForFinalization) {
        resolve(result.txHash.toHex())
      }
      if (result.isFinalized) {
        resolve(result.txHash.toHex())
      }
    })
  })
}

export const deployInvariant = async (
  api: ApiPromise,
  account: IKeyringPair,
  initFee: Percentage,
  network: Network
): Promise<Invariant> => {
  return Invariant.getContract(
    api,
    account,
    null,
    DEFAULT_REF_TIME,
    DEFAULT_PROOF_SIZE,
    initFee,
    network
  )
}

export const deployPSP22 = async (
  api: ApiPromise,
  account: IKeyringPair,
  supply: bigint,
  name: string,
  symbol: string,
  decimals: bigint,
  network: Network
): Promise<PSP22> => {
  return PSP22.getContract(
    api,
    network,
    null,
    DEFAULT_REF_TIME,
    DEFAULT_PROOF_SIZE,
    account,
    supply,
    name,
    symbol,
    decimals
  )
}

export const deployWrappedAZERO = async (
  api: ApiPromise,
  account: IKeyringPair,
  network: Network
): Promise<WrappedAZERO> => {
  return WrappedAZERO.getContract(api, account, null, DEFAULT_REF_TIME, DEFAULT_PROOF_SIZE, network)
}

export const convertObj = <T>(obj: T): T => {
  const newObj: { [key: string]: any } = {}

  Object.entries(obj as { [key: string]: any }).forEach(([key, value]) => {
    newObj[key] = value

    if (typeof value === 'number' || (typeof value === 'string' && value.startsWith('0x'))) {
      newObj[key] = BigInt(value)
    }

    if (typeof value.v === 'number' || (typeof value.v === 'string' && value.v.startsWith('0x'))) {
      newObj[key] = { v: BigInt(value.v) }
    }

    if (typeof value === 'object' && value.v === undefined) {
      newObj[key] = convertObj(value)
    }

    if (value.constructor === Array) {
      newObj[key] = convertArr(value)
    }
  })

  return newObj as T
}

export const convertArr = (arr: any[]): any[] => {
  return arr.map(value => {
    if (typeof value === 'number' || (typeof value === 'string' && value.startsWith('0x'))) {
      return BigInt(value)
    }

    if (typeof value.v === 'number' || (typeof value.v === 'string' && value.v.startsWith('0x'))) {
      return { v: BigInt(value.v) }
    }

    if (typeof value === 'object' && value.v === undefined) {
      return convertObj(value)
    }

    if (value.constructor === Array) {
      return convertArr(value)
    }

    return value
  })
}

export const assertThrowsAsync = async (fn: Promise<any>, word?: InvariantError | InvariantTx) => {
  try {
    await fn
  } catch (e: any) {
    if (word) {
      const err = e.toString()
      console.log(err)
      const regex = new RegExp(`${word}$`)
      if (!regex.test(err)) {
        console.log(err)
        throw new Error('Invalid Error message')
      }
    }
    return
  }
  throw new Error('Function did not throw error')
}
