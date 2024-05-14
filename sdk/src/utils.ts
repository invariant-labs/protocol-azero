/* eslint-disable no-case-declarations */

import { ApiPromise, SubmittableResult, WsProvider } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { SubmittableExtrinsic } from '@polkadot/api/promise/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { getSubstrateChain, initPolkadotJs as initApi } from '@scio-labs/use-inkathon'
import {
  FeeTier,
  LiquidityTick,
  Percentage,
  Pool,
  PoolKey,
  Position,
  Price,
  SqrtPrice,
  Tick,
  TokenAmount,
  _calculateFee,
  _newFeeTier,
  _newPoolKey,
  calculateAmountDelta,
  calculateAmountDeltaResult,
  getMaxChunk,
  getPercentageDenominator,
  getSqrtPriceDenominator
} from 'invariant-a0-wasm/invariant_a0_wasm.js'
import { abi as invariantAbi } from './abis/invariant.js'
import { abi as PSP22Abi } from './abis/psp22.js'
import { abi as wrappedAZEROAbi } from './abis/wrapped-azero.js'
import { MAINNET, TESTNET } from './consts.js'
import { Network } from './network.js'
import { EventTxResult, LiquidityBreakpoint, Query, Tx, TxResult } from './schema.js'

export const initPolkadotApi = async (network: Network, ws?: string): Promise<ApiPromise> => {
  if (network === Network.Local) {
    const wsProvider = new WsProvider(ws)
    const api = await ApiPromise.create({ provider: wsProvider })
    await api.isReady
    return api
  } else if (network === Network.Testnet) {
    const chain = getSubstrateChain(TESTNET)

    if (!chain) {
      throw new Error('chain not found')
    }

    const { api } = await initApi(chain, { noInitWarn: true })
    return api
  } else if (network === Network.Mainnet) {
    const chain = getSubstrateChain(MAINNET)

    if (!chain) {
      throw new Error('chain not found')
    }

    const { api } = await initApi(chain, { noInitWarn: true })
    return api
  } else {
    throw new Error('invalid network')
  }
}

export async function sendQuery(
  contract: ContractPromise,
  gasLimit: WeightV2,
  storageDepositLimit: number | null,
  userAddress: string,
  message: Query | Tx,
  data: any[]
): Promise<any> {
  const { result, output } = await contract.query[message](
    userAddress,
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

export function createTx(
  contract: ContractPromise,
  gasLimit: WeightV2,
  storageDepositLimit: number | null,
  value: bigint,
  message: Tx,
  data: any[]
): SubmittableExtrinsic {
  if (!contract) {
    throw new Error('contract not loaded')
  }

  return contract.tx[message](
    {
      gasLimit,
      storageDepositLimit,
      value
    },
    ...data
  )
}

export async function handleTxResult(
  result: SubmittableResult,
  resolve: any,
  reject: any,
  waitForFinalization: boolean = true,
  block: boolean = true
) {
  if (!block) {
    resolve({
      hash: result.txHash.toHex(),
      events: parseEvents((result as any).contractEvents || []) as []
    })
  }

  if (result.isError || result.dispatchError) {
    reject(new Error(result.dispatchError?.toString() || 'error'))
  }

  if (result.isCompleted && !waitForFinalization) {
    resolve({
      hash: result.txHash.toHex(),
      events: parseEvents((result as any).contractEvents || []) as []
    })
  }

  if (result.isFinalized) {
    resolve({
      hash: result.txHash.toHex(),
      events: parseEvents((result as any).contractEvents || []) as []
    })
  }
}

export async function sendTx(
  tx: SubmittableExtrinsic,
  waitForFinalization: boolean = true,
  block: boolean = true
): Promise<EventTxResult<any> | TxResult> {
  return new Promise(async (resolve, reject) => {
    await tx.send(result => {
      handleTxResult(result, resolve, reject, waitForFinalization, block)
    })
  })
}

export async function signAndSendTx(
  tx: SubmittableExtrinsic,
  signer: IKeyringPair,
  waitForFinalization: boolean = true,
  block: boolean = true
): Promise<EventTxResult<any> | TxResult> {
  return new Promise(async (resolve, reject) => {
    await tx.signAndSend(signer, result => {
      handleTxResult(result, resolve, reject, waitForFinalization, block)
    })
  })
}

export async function createSignAndSendTx(
  contract: ContractPromise,
  gasLimit: WeightV2,
  storageDepositLimit: number | null,
  value: bigint,
  signer: IKeyringPair,
  message: Tx,
  data: any[],
  waitForFinalization: boolean = true,
  block: boolean = true
): Promise<EventTxResult<any> | TxResult> {
  const tx = createTx(contract, gasLimit, storageDepositLimit, value, message, data)

  return await signAndSendTx(tx, signer, waitForFinalization, block)
}

export const newPoolKey = (token0: string, token1: string, feeTier: FeeTier): PoolKey => {
  return parse(
    _newPoolKey(token0, token1, _newFeeTier(feeTier.fee, integerSafeCast(feeTier.tickSpacing)))
  )
}

export const newFeeTier = (fee: Percentage, tickSpacing: bigint): FeeTier => {
  return parse(_newFeeTier(fee, integerSafeCast(tickSpacing)))
}

export const parseEvent = (event: { [key: string]: any }) => {
  const eventObj: { [key: string]: any } = {}

  for (const [index, arg] of event.args.entries()) {
    eventObj[event.event.args[index].name] = arg.toPrimitive()
  }

  return parse(eventObj)
}

export const parseEvents = (events: { [key: string]: any }[]) => {
  return events.map(event => parseEvent(event))
}

let nodeModules: typeof import('./node')

const loadNodeModules = async () => {
  if (typeof window !== 'undefined') {
    throw new Error('cannot load node modules in a browser environment')
  }

  await import('./node')
    .then(node => {
      nodeModules = node
    })
    .catch(error => {
      console.error('error while loading node modules:', error)
    })
}

export const getDeploymentData = async (
  contractName: string
): Promise<{ abi: any; wasm: Buffer }> => {
  await loadNodeModules()
  const __dirname = new URL('.', import.meta.url).pathname

  try {
    const abi = JSON.parse(
      await nodeModules.readFile(
        nodeModules.join(__dirname, `../contracts/${contractName}/${contractName}.json`),
        'utf-8'
      )
    )
    const wasm = await nodeModules.readFile(
      nodeModules.join(__dirname, `../contracts/${contractName}/${contractName}.wasm`)
    )

    return { abi, wasm }
  } catch (error) {
    throw new Error(`${contractName}.json or ${contractName}.wasm not found`)
  }
}

export const getAbi = async (contractName: string): Promise<any> => {
  switch (contractName) {
    case 'invariant':
      return JSON.parse(invariantAbi)
    case 'psp22':
      return JSON.parse(PSP22Abi)
    case 'wrapped-azero':
      return JSON.parse(wrappedAZEROAbi)
    default:
      throw new Error('contract not found')
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
  const price = sqrtPriceToPrice(sqrtPrice)
  const priceWithSlippage = price * multiplier * getPercentageDenominator()
  const sqrtPriceWithSlippage = priceToSqrtPrice(priceWithSlippage) / getPercentageDenominator()

  return sqrtPriceWithSlippage
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

export const calculateFee = (
  pool: Pool,
  position: Position,
  lowerTick: Tick,
  upperTick: Tick
): [TokenAmount, TokenAmount] => {
  return _calculateFee(
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
export const calculateTokenAmounts = (
  pool: Pool,
  position: Position
): calculateAmountDeltaResult => {
  return _calculateTokenAmounts(pool, position, false)
}

export const _calculateTokenAmounts = (
  pool: Pool,
  position: Position,
  sign: boolean
): calculateAmountDeltaResult => {
  return calculateAmountDelta(
    pool.currentTickIndex,
    pool.sqrtPrice,
    position.liquidity,
    sign,
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

export const integerSafeCast = (value: bigint): number => {
  if (value > BigInt(Number.MAX_SAFE_INTEGER) || value < BigInt(Number.MIN_SAFE_INTEGER)) {
    throw new Error('Integer value is outside the safe range for Numbers')
  }
  return Number(value)
}

export const constructTickmap = (initializedChunks: bigint[][], tickSpacing: bigint): bigint[] => {
  console.log(initializedChunks)
  const maxChunk = getMaxChunk(tickSpacing)
  const tickmap = new Array<bigint>(integerSafeCast(maxChunk + 1n)).fill(0n)

  for (const [chunkIndex, value] of initializedChunks) {
    tickmap[integerSafeCast(chunkIndex)] = value
  }
  return tickmap
}

export const sqrtPriceToPrice = (sqrtPrice: SqrtPrice): Price => {
  return (sqrtPrice * sqrtPrice) / getSqrtPriceDenominator()
}

export const priceToSqrtPrice = (price: Price): SqrtPrice => {
  return sqrt(price * getSqrtPriceDenominator())
}

export const calculateLiquidityBreakpoints = (
  ticks: (Tick | LiquidityTick)[]
): LiquidityBreakpoint[] => {
  let currentLiquidity = 0n

  return ticks.map(tick => {
    currentLiquidity = currentLiquidity + tick.liquidityChange * (tick.sign ? 1n : -1n)
    return {
      liquidity: currentLiquidity,
      index: tick.index
    }
  })
}
