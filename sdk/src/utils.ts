/* eslint-disable no-case-declarations */

import {
  CalculateSwapResult,
  FeeTier,
  InvariantError,
  Liquidity,
  LiquidityTick,
  Percentage,
  Pool,
  PoolKey,
  Position,
  Price,
  SqrtPrice,
  Tick,
  Tickmap,
  TokenAmount,
  _calculateFee,
  _newFeeTier,
  _newPoolKey,
  positionToTick as _positionToTick,
  simulateInvariantSwap as _simulateInvariantSwap,
  alignTickToSpacing,
  calculateAmountDelta,
  calculateAmountDeltaResult,
  calculateTick,
  getMaxChunk,
  getMaxTickCross,
  getPercentageDenominator,
  getSqrtPriceDenominator,
  tickIndexToPosition,
  toPercentage,
  getMinTick as _getMinTick,
  getMaxTick as _getMaxTick,
  _toFeeGrowth
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { ApiPromise, SubmittableResult, WsProvider } from '@polkadot/api'
import { ContractPromise } from '@polkadot/api-contract'
import { SubmittableExtrinsic } from '@polkadot/api/promise/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { getSubstrateChain, initPolkadotJs as initApi } from '@scio-labs/use-inkathon'
import { abi as invariantAbi } from './abis/invariant.js'
import { abi as PSP22Abi } from './abis/psp22.js'
import { abi as wrappedAZEROAbi } from './abis/wrapped-azero.js'
import { CONCENTRATION_FACTOR, MAINNET, TESTNET } from './consts.js'
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
  message: Query | Tx,
  data: any[],
  userAddress: string = ''
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

// TODO: to REMOVE
export async function sendAndDebugTx(
  tx: SubmittableExtrinsic,
  api: ApiPromise,
  waitForFinalization: boolean = true,
  block: boolean = true
): Promise<EventTxResult<any> | TxResult> {
  return new Promise(async (resolve, reject) => {
    await tx.send(result => {
      result.events
        .filter(({ event }) => api.events.system.ExtrinsicFailed.is(event))
        .forEach(
          ({
            event: {
              data: [error]
            }
          }) => {
            // @ts-expect-error not typed error
            if (error.isModule) {
              // for module errors, we have the section indexed, lookup
              // @ts-expect-error not typed error
              const decoded = api.registry.findMetaError(error.asModule)
              const { docs, method, section } = decoded

              console.log(`${section}.${method}: ${docs.join(' ')}`)
            } else {
              // Other, CannotLookup, BadOrigin, no extra info
              console.log(error.toString())
            }
          }
        )
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

let nodeModules: typeof import('./node.js')

const loadNodeModules = async () => {
  if (typeof window !== 'undefined') {
    throw new Error('cannot load node modules in a browser environment')
  }

  await import('./node.js')
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
  if (slippage === 0n) {
    return sqrtPrice
  }

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
    lowerTick.feeGrowthOutsideX.toString(),
    lowerTick.feeGrowthOutsideY.toString(),
    upperTick.index,
    upperTick.feeGrowthOutsideX.toString(),
    upperTick.feeGrowthOutsideY.toString(),
    pool.currentTickIndex,
    pool.feeGrowthGlobalX.toString(),
    pool.feeGrowthGlobalY.toString(),
    position.feeGrowthInsideX.toString(),
    position.feeGrowthInsideY.toString(),
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

// deserialize functions should be used after calling parse
export const deserializeArrayToU256 = (value: [bigint, bigint, bigint, bigint]) => {
  let ret = 0n
  for (let i = 0n; i < 4n; i++) {
    ret += value[Number(i)] * (1n << (64n * i))
  }
  return ret
}

export const deserializeTickFromContract = (value: any) => {
  value.feeGrowthOutsideX = deserializeArrayToU256(value.feeGrowthOutsideX)
  value.feeGrowthOutsideY = deserializeArrayToU256(value.feeGrowthOutsideY)
  return value
}

export const deserializePoolFromContract = (value: any) => {
  value.feeGrowthGlobalX = deserializeArrayToU256(value.feeGrowthGlobalX)
  value.feeGrowthGlobalY = deserializeArrayToU256(value.feeGrowthGlobalY)
  return value
}

export const deserializePositionFromContract = (value: any) => {
  value.feeGrowthInsideX = deserializeArrayToU256(value.feeGrowthInsideX)
  value.feeGrowthInsideY = deserializeArrayToU256(value.feeGrowthInsideY)
  return value
}

// deserialize functions should be used before passing FeeGrowth to wasm via a struct or just a raw variable
export const serializeU256 = (value: bigint) => {
  return value.toString()
}

export const serializeTick = (value: any) => {
  value.feeGrowthOutsideX = serializeU256(value.feeGrowthOutsideX)
  value.feeGrowthOutsideY = serializeU256(value.feeGrowthOutsideY)
  return value
}

export const serializePool = (value: any) => {
  value.feeGrowthGlobalX = serializeU256(value.feeGrowthGlobalX)
  value.feeGrowthGlobalY = serializeU256(value.feeGrowthGlobalY)
  return value
}

export const serializePosition = (value: any) => {
  value.feeGrowthInsideX = serializeU256(value.feeGrowthInsideX)
  value.feeGrowthInsideY = serializeU256(value.feeGrowthInsideY)
  return value
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

export const extractError = (err: any) => {
  const error = Object.keys(err)[0]
  const parsedError = error[0].toUpperCase() + error.slice(1)
  return InvariantError[parsedError as any]
}

export function getActiveBitsCount64(num: bigint) {
  let activeBits = 0n
  let bit = 0n

  while (bit < 64) {
    if (num & (1n << bit)) {
      activeBits += 1n
    }
    bit += 1n
  }

  return activeBits
}

export function lowestActiveBit(num: bigint) {
  let bit = 0n

  while (bit < 64) {
    if (num & (1n << bit)) {
      return bit
    }
    bit += 1n
  }

  return bit
}

export function highestActiveBit(num: bigint) {
  let bit = 63n

  while (bit >= 0n) {
    if (num & (1n << bit)) {
      return bit
    }
    bit -= 1n
  }

  return bit
}

export function simulateInvariantSwap(
  tickmap: Tickmap,
  feeTier: FeeTier,
  pool: Pool,
  ticks: LiquidityTick[],
  xToY: boolean,
  amountIn: TokenAmount,
  byAmountIn: boolean,
  sqrtPriceLimit: SqrtPrice
): CalculateSwapResult {
  return _simulateInvariantSwap(
    tickmap,
    feeTier,
    serializePool(pool),
    ticks,
    xToY,
    amountIn,
    byAmountIn,
    sqrtPriceLimit
  )
}

export function positionToTick(chunkIndex: bigint, bit: bigint, tickSpacing: bigint): bigint {
  return BigInt(_positionToTick(chunkIndex, bit, tickSpacing))
}

export function filterTicks<T extends Tick | LiquidityTick>(
  ticks: T[],
  tickIndex: bigint,
  xToY: boolean
): T[] {
  const filteredTicks = new Array(...ticks)
  const maxTicksCross = getMaxTickCross()
  let tickCount = 0

  for (const [index, tick] of filteredTicks.entries()) {
    if (tickCount >= maxTicksCross) {
      break
    }

    if (xToY) {
      if (tick.index > tickIndex) {
        filteredTicks.splice(index, 1)
      }
    } else {
      if (tick.index < tickIndex) {
        filteredTicks.splice(index, 1)
      }
    }
    tickCount++
  }

  return ticks
}

export function filterTickmap(
  tickmap: Tickmap,
  tickSpacing: bigint,
  index: bigint,
  xToY: boolean
): Tickmap {
  const filteredTickmap = new Map(tickmap.bitmap)
  const [currentChunkIndex] = tickIndexToPosition(index, tickSpacing)
  const maxTicksCross = getMaxTickCross()
  let tickCount = 0
  for (const [chunkIndex] of filteredTickmap) {
    if (tickCount >= maxTicksCross) {
      break
    }

    if (xToY) {
      if (chunkIndex > currentChunkIndex) {
        filteredTickmap.delete(chunkIndex)
      }
    } else {
      if (chunkIndex < currentChunkIndex) {
        filteredTickmap.delete(chunkIndex)
      }
    }
    tickCount++
  }

  return { bitmap: filteredTickmap }
}

export const delay = (delayMs: number) => {
  return new Promise(resolve => setTimeout(resolve, delayMs))
}
export const calculateFeeTierWithLinearRatio = (tickCount: bigint): FeeTier => {
  return newFeeTier(tickCount * toPercentage(1n, 4n), tickCount)
}

export const assert = (condition: boolean, message?: string) => {
  if (!condition) {
    throw new Error(message || 'assertion failed')
  }
}

export const calculateConcentration = (tickSpacing: number, minimumRange: number, n: number) => {
  const concentration = 1 / (1 - Math.pow(1.0001, (-tickSpacing * (minimumRange + 2 * n)) / 4))
  return concentration / CONCENTRATION_FACTOR
}

export const calculateTickDelta = (
  tickSpacing: number,
  minimumRange: number,
  concentration: number
) => {
  const base = Math.pow(1.0001, -(tickSpacing / 4))
  const logArg =
    (1 - 1 / (concentration * CONCENTRATION_FACTOR)) /
    Math.pow(1.0001, (-tickSpacing * minimumRange) / 4)

  return Math.ceil(Math.log(logArg) / Math.log(base) / 2)
}

export const getConcentrationArray = (
  tickSpacing: number,
  minimumRange: number,
  currentTick: number
): number[] => {
  const concentrations: number[] = []
  let counter = 0
  let concentration = 0
  let lastConcentration = calculateConcentration(tickSpacing, minimumRange, counter) + 1
  let concentrationDelta = 1

  while (concentrationDelta >= 1) {
    concentration = calculateConcentration(tickSpacing, minimumRange, counter)
    concentrations.push(concentration)
    concentrationDelta = lastConcentration - concentration
    lastConcentration = concentration
    counter++
  }
  concentration = Math.ceil(concentrations[concentrations.length - 1])

  while (concentration > 1) {
    concentrations.push(concentration)
    concentration--
  }
  const maxTick = integerSafeCast(alignTickToSpacing(getMaxTick(1n), tickSpacing))
  if ((minimumRange / 2) * tickSpacing > maxTick - Math.abs(currentTick)) {
    throw new Error(String(InvariantError.TickLimitReached))
  }
  const limitIndex =
    (maxTick - Math.abs(currentTick) - (minimumRange / 2) * tickSpacing) / tickSpacing

  return concentrations.slice(0, limitIndex)
}

export const calculateTokenAmountsWithSlippage = (
  tickSpacing: bigint,
  currentSqrtPrice: SqrtPrice,
  liquidity: Liquidity,
  lowerTickIndex: bigint,
  upperTickIndex: bigint,
  slippage: Percentage,
  roundingUp: boolean
): [bigint, bigint] => {
  const lowerBound = calculateSqrtPriceAfterSlippage(currentSqrtPrice, slippage, false)
  const upperBound = calculateSqrtPriceAfterSlippage(currentSqrtPrice, slippage, true)

  const currentTickIndex = calculateTick(currentSqrtPrice, tickSpacing)

  const [lowerX, lowerY] = calculateAmountDelta(
    currentTickIndex,
    lowerBound,
    liquidity,
    roundingUp,
    upperTickIndex,
    lowerTickIndex
  )
  const [upperX, upperY] = calculateAmountDelta(
    currentTickIndex,
    upperBound,
    liquidity,
    roundingUp,
    upperTickIndex,
    lowerTickIndex
  )

  const x = lowerX > upperX ? lowerX : upperX
  const y = lowerY > upperY ? lowerY : upperY
  return [x, y]
}

export const getMinTick = (tickSpacing: bigint): bigint => {
  return BigInt(_getMinTick(tickSpacing))
}

export const getMaxTick = (tickSpacing: bigint): bigint => {
  return BigInt(_getMaxTick(tickSpacing))
}

export const toFeeGrowth = (value: bigint, scale: bigint): bigint => {
  return BigInt(_toFeeGrowth(value, scale))
}

export const getCodeHash = async (api: ApiPromise, contractAddress: string): Promise<string> => {
  const result = await api.query.contracts.contractInfoOf(contractAddress)
  const { codeHash } = JSON.parse(JSON.stringify(result))
  return codeHash
}
