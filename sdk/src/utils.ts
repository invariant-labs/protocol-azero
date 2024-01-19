import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { getSubstrateChain } from '@scio-labs/use-inkathon/chains'
import { getBalance, initPolkadotJs as initApi } from '@scio-labs/use-inkathon/helpers'
import { readFile } from 'fs/promises'
import {
  FeeTier,
  Percentage,
  Pool,
  PoolKey,
  Position,
  SqrtPrice,
  Tick,
  TokenAmounts,
  _newFeeTier,
  _newPoolKey,
  _simulateUnclaimedFees,
  getPercentageDenominator,
  getSqrtPriceDenominator,
  wrappedCalculateTokenAmounts
} from 'math/math.js'
import { Network } from './network.js'
import { Query, Tx, TxResult } from './schema.js'

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

export async function sendQuery(
  contract: ContractPromise,
  gasLimit: WeightV2,
  storageDepositLimit: number | null,
  signer: IKeyringPair,
  message: Query | Tx,
  data: any[]
): Promise<any> {
  const { result, output } = await contract.query[message](
    signer.address,
    {
      gasLimit: gasLimit,
      storageDepositLimit: storageDepositLimit
    },
    ...data
  )

  if (result.isOk && output) {
    return parse(JSON.parse(output.toString()).ok)
  } else {
    throw new Error(result.asErr.toHuman()?.toString())
  }
}

export async function sendTx(
  contract: ContractPromise,
  gasLimit: WeightV2,
  storageDepositLimit: number | null,
  value: bigint,
  signer: IKeyringPair,
  message: Tx,
  data: any[],
  waitForFinalization: boolean = true,
  block: boolean = true
): Promise<TxResult> {
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

  return new Promise<TxResult>(async (resolve, reject) => {
    await call.signAndSend(signer, result => {
      if (!block) {
        resolve({
          hash: result.txHash.toHex(),
          events: parseEvents((result as any).contractEvents || [])
        })
      }

      if (result.isError || result.dispatchError) {
        reject(new Error(message))
      }

      if (result.isCompleted && !waitForFinalization) {
        resolve({
          hash: result.txHash.toHex(),
          events: parseEvents((result as any).contractEvents || [])
        })
      }

      if (result.isFinalized) {
        resolve({
          hash: result.txHash.toHex(),
          events: parseEvents((result as any).contractEvents || [])
        })
      }
    })
  })
}

export const printBalance = async (api: ApiPromise, account: IKeyringPair) => {
  const network = (await api.rpc.system.chain())?.toString() || ''
  const version = (await api.rpc.system.version())?.toString() || ''
  const balance = await getBalance(api, account.address)

  console.log(`network: ${network} (${version})`)
  console.log(`account: ${account.address} (${balance.balanceFormatted})\n`)
}

export const newPoolKey = (token0: string, token1: string, feeTier: FeeTier): PoolKey => {
  return parse(_newPoolKey(token0, token1, _newFeeTier(feeTier.fee, Number(feeTier.tickSpacing))))
}

export const newFeeTier = (fee: Percentage, tickSpacing: bigint): FeeTier => {
  return parse(_newFeeTier(fee, Number(tickSpacing)))
}

export const getEnvAccount = async (keyring: Keyring): Promise<IKeyringPair> => {
  const accountUri = process.env.ACCOUNT_URI

  if (!accountUri) {
    throw new Error('invalid account uri')
  }

  return keyring.addFromUri(accountUri)
}

export const parseEvent = (event: { [key: string]: any }) => {
  const eventObj: { [key: string]: any } = {}

  for (let i = 0; i < event.args.length; i++) {
    eventObj[event.event.args[i].name] = event.args[i].toPrimitive()
  }

  return parse(eventObj)
}

export const parseEvents = (events: { [key: string]: any }[]) => {
  return events.map(event => parseEvent(event))
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

const sqrt = (value: bigint): bigint => {
  if (value < 0n) {
    throw 'square root of negative numbers is not supported'
  }

  if (value < 2n) {
    return value
  }

  return newtonIteration(value, 1n)
}

const newtonIteration = (n: bigint, x0: bigint): bigint => {
  const x1 = (n / x0 + x0) >> 1n
  if (x0 === x1 || x0 === x1 - 1n) {
    return x0
  }
  return newtonIteration(n, x1)
}

export const calculateSqrtPriceAfterSlippage = (
  sqrtPrice: SqrtPrice,
  slippage: Percentage,
  up: boolean
): SqrtPrice => {
  const multiplier = getPercentageDenominator() + (up ? slippage : -slippage)

  return (
    sqrt(
      ((sqrtPrice * sqrtPrice) / getSqrtPriceDenominator()) *
        multiplier *
        getSqrtPriceDenominator() *
        getPercentageDenominator()
    ) / getPercentageDenominator()
  )
}

export const calculatePriceImpact = (
  startingSqrtPrice: SqrtPrice,
  endingSqrtPrice: SqrtPrice
): Percentage => {
  const startingPrice = startingSqrtPrice * startingSqrtPrice
  const endingPrice = endingSqrtPrice * endingSqrtPrice
  const diff = startingPrice - endingPrice

  const nominator = diff > 0n ? diff : -diff
  const denominator = startingPrice > endingPrice ? startingPrice : endingPrice

  return (nominator * getPercentageDenominator()) / denominator
}

export const simulateUnclaimedFees = (
  pool: Pool,
  position: Position,
  lowerTick: Tick,
  upperTick: Tick
): TokenAmounts => {
  return _simulateUnclaimedFees(
    lowerTick.index,
    lowerTick.feeGrowthOutsideX,
    lowerTick.feeGrowthOutsideY,
    upperTick.index,
    upperTick.feeGrowthOutsideX,
    upperTick.feeGrowthOutsideY,
    pool.currentTickIndex,
    pool.feeGrowthGlobalX,
    pool.feeGrowthGlobalY,
    position.feeGrowthInsideX,
    position.feeGrowthInsideY,
    position.liquidity
  )
}
export const calculateTokenAmounts = (pool: Pool, position: Position): TokenAmounts => {
  return wrappedCalculateTokenAmounts(
    pool.currentTickIndex,
    pool.sqrtPrice,
    position.liquidity,
    position.upperTickIndex,
    position.lowerTickIndex
  )
}

export const parse = (value: any) => {
  if (isArray(value)) {
    return value.map((element: any) => parse(element))
  }

  if (isObject(value)) {
    const newValue: { [key: string]: any } = {}

    Object.entries(value as { [key: string]: any }).forEach(([key, value]) => {
      newValue[key] = parse(value)
    })

    return newValue
  }

  if (isBoolean(value)) {
    return value
  }

  try {
    return BigInt(value)
  } catch (e) {
    return value
  }
}

const isBoolean = (value: any): boolean => {
  return typeof value === 'boolean'
}

const isArray = (value: any): boolean => {
  return Array.isArray(value)
}

const isObject = (value: any): boolean => {
  return typeof value === 'object' && value !== null
}
