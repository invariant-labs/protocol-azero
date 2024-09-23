import {
  getChunkSize,
  getFeeGrowthDenominator,
  getFeeGrowthScale,
  getFixedPointDenominator,
  getFixedPointScale,
  getGlobalMaxSqrtPrice,
  getGlobalMinSqrtPrice,
  getLiquidityDenominator,
  getLiquidityScale,
  getLiquidityTicksLimit,
  getMaxPoolKeysReturned,
  getMaxPoolPairsReturned,
  getMaxSwapSteps,
  getMaxTickmapQuerySize,
  getPercentageDenominator,
  getPercentageScale,
  getPositionsEntriesLimit,
  getPriceDenominator,
  getPriceScale,
  getSecondsPerLiquidityDenominator,
  getSecondsPerLiquidityScale,
  getSqrtPriceDenominator,
  getSqrtPriceScale,
  getTickSearchRange,
  getTokenAmountDenominator,
  getTokenAmountScale
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Network } from './network.js'

export const MAX_REF_TIME = 259058343000
export const DEFAULT_REF_TIME = 1250000000000
export const DEFAULT_PROOF_SIZE = 1250000000000
export const CONCENTRATION_FACTOR = 1.00001526069123
export const TESTNET = 'alephzero-testnet'
export const MAINNET = 'alephzero'
export const DEFAULT_LOCAL = 'ws://127.0.0.1:9944'

export const WAZERO_ADDRESS = {
  [Network.Testnet]: '5EFDb7mKbougLtr5dnwd5KDfZ3wK55JPGPLiryKq4uRMPR46',
  [Network.Mainnet]: '5CtuFVgEUz13SFPVY6s2cZrnLDEkxQXc19aXrNARwEBeCXgg',
  [Network.Local]: ''
}

export const INVARIANT_ADDRESS = {
  [Network.Testnet]: '5E6xib2HZi2ZpWmuDRiBYW9HDjTuSJA69eE9ncorTnj6XQKm',
  [Network.Mainnet]: '5GTv4yqNS48e5QJ9fr14ck6i2gpn1gFvL7MNnQadFtEDYALF',
  [Network.Local]: ''
}
export const BTC_ADDRESS = {
  [Network.Testnet]: '5GM2m3FKEwrDNz9Z8m3G2bQiPg7Gqbjtdx1regWqdDXQG79a',
  [Network.Mainnet]: '5D6Lga7jXKAx4kFHprP1AbPc3zrvbBZvZBUHMTH4LrdsWdkG',
  [Network.Local]: ''
}
export const ETH_ADDRESS = {
  [Network.Testnet]: '5GxQ6xxDD1szDNLF9tyNfJyGFhi2png2kiWodcKYxypNbzCb',
  [Network.Mainnet]: '5F8o46LxMg3LF26DtCZWV8fwinSg5sYs2sBi3XypZWsTYQKB',
  [Network.Local]: ''
}
export const USDC_ADDRESS = {
  [Network.Testnet]: '5F5QApSFE5iSvJ8M6zZ26i2iHnrmZ4TL6gy3sA8u8crm1eRU',
  [Network.Mainnet]: '5Dj9Jmk2GnLSuXaEZN8WjUxB9zWNKa75tKYdpsUQFqzaCJ6Y',
  [Network.Local]: ''
}
export const USDT_ADDRESS = {
  [Network.Testnet]: '5CTeupkDMtraJXU33ugZ1ueJmtR7pgbuPhR5So3Qo7BiyMfB',
  [Network.Mainnet]: '5DgtfRBJjEqwJqLYPxgesmNsTuxcxhR2xCGaPEpYBDq4LyhJ',
  [Network.Local]: ''
}
export const SOL_ADDRESS = {
  [Network.Testnet]: '5HJ1QkdZEDKaYxGbdaaNBTwQXNPxeJHJ9hhp4ddBP3aeDVBq',
  [Network.Mainnet]: '5GVuwRfGo5e5YRvBpq1rsQd3KKfCLWppHUuXyDsj89ASuNuq',
  [Network.Local]: ''
}

export const FEE_GROWTH_DENOMINATOR = getFeeGrowthDenominator()
export const FIXED_POINT_DENOMINATOR = getFixedPointDenominator()
export const LIQUIDITY_DENOMINATOR = getLiquidityDenominator()
export const PERCENTAGE_DENOMINATOR = getPercentageDenominator()
export const PRICE_DENOMINATOR = getPriceDenominator()
export const SECONDS_PER_LIQUIDITY_DENOMINATOR = getSecondsPerLiquidityDenominator()
export const SQRT_PRICE_DENOMINATOR = getSqrtPriceDenominator()
export const TOKEN_AMOUNT_DENOMINATOR = getTokenAmountDenominator()

export const FEE_GROWTH_SCALE = getFeeGrowthScale()
export const FIXED_POINT_SCALE = getFixedPointScale()
export const LIQUIDITY_SCALE = getLiquidityScale()
export const PERCENTAGE_SCALE = getPercentageScale()
export const PRICE_SCALE = getPriceScale()
export const SECONDS_PER_LIQUIDITY_SCALE = getSecondsPerLiquidityScale()
export const SQRT_PRICE_SCALE = getSqrtPriceScale()
export const TOKEN_AMOUNT_SCALE = getTokenAmountScale()

export const MAX_SQRT_PRICE = getGlobalMaxSqrtPrice()
export const MIN_SQRT_PRICE = getGlobalMinSqrtPrice()
export const CHUNK_SIZE = getChunkSize()
export const MAX_TICKMAP_QUERY_SIZE = getMaxTickmapQuerySize()
export const MAX_SWAP_STEPS = getMaxSwapSteps()
export const LIQUIDITY_TICKS_LIMIT = getLiquidityTicksLimit()
export const MAX_POOL_KEYS_RETURNED = getMaxPoolKeysReturned()
export const MAX_POOL_PAIRS_RETURNED = getMaxPoolPairsReturned()
export const POSITIONS_ENTRIES_LIMIT = getPositionsEntriesLimit()
export const SEARCH_RANGE = getTickSearchRange()
