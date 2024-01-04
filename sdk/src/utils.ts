import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { getSubstrateChain } from '@scio-labs/use-inkathon/chains'
import { getBalance, initPolkadotJs as initApi } from '@scio-labs/use-inkathon/helpers'
import { readFile } from 'fs/promises'
import { Percentage } from 'math'
import { Invariant } from './invariant.js'
import { Network } from './network.js'
import { PSP22 } from './psp22.js'
import { InvariantQuery, InvariantTx, PSP22Query, PSP22Tx, WrappedAZEROTx } from './schema.js'

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
  message: InvariantQuery | PSP22Query,
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
  message: InvariantTx | PSP22Tx | WrappedAZEROTx,
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
        console.log('ERROR BRACKETS')
        const err = new Error(`Tx: ${message} reverted`)
        return reject(err)
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
  initFee: Percentage
): Promise<Invariant> => {
  const invariantData = await getDeploymentData('invariant')
  const invariant = new Invariant(api, Network.Local)

  const invariantDeploy = await invariant.deploy(
    account,
    invariantData.abi,
    invariantData.wasm,
    initFee
  )
  await invariant.load(invariantDeploy.address, invariantData.abi)

  return invariant
}

export const deployPSP22 = async (
  api: ApiPromise,
  account: IKeyringPair,
  supply: bigint,
  name: string = 'Coin',
  symbol: string = 'COIN',
  decimals: number = 12
): Promise<PSP22> => {
  const tokenData = await getDeploymentData('psp22')
  const token = new PSP22(api, Network.Local)

  const tokenDeploy = await token.deploy(
    account,
    tokenData.abi,
    tokenData.wasm,
    supply,
    name,
    symbol,
    decimals
  )
  token.address = tokenDeploy.address

  await token.load(tokenDeploy.address, tokenData.abi)
  return token
}

export async function assertThrowsAsync(fn: Promise<any>, word?: string) {
  try {
    await fn
  } catch (e: any) {
    const err = e.toString()
    if (word) {
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
