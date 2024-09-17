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
  [Network.Testnet]: '5FrYjo3Y7CmzvLveD96F66SMYJdcBPcJh9kSxq214y4LTXdj',
  [Network.Mainnet]: '5CvocBcChFccUkNGZpYf1mThQQDaY7ZxXEmdTXbTLqt1SaYQ',
  [Network.Local]: ''
}
export const BTC_ADDRESS = {
  [Network.Testnet]: '5DHLwE9znALML2kaKbHz1X4W243ae3ijqCQjbVoVwVpsEGnp',
  [Network.Mainnet]: '5HW9QeCifdKt8gXwXVSE8z56njDQBhGfses1KJNFL68qius9',
  [Network.Local]: ''
}
export const ETH_ADDRESS = {
  [Network.Testnet]: '5Cqosz5NxwvuudycTmphjmjBaGnUf25wnu9CSbDAR2616j7h',
  [Network.Mainnet]: '5EEzffpXkfYKkdtmqNh9UNctYTjmbi9GfKKAWRTKKFh6F1FU',
  [Network.Local]: ''
}
export const USDC_ADDRESS = {
  [Network.Testnet]: '5HGDNuKD1knU8ZQ3zQXZCMniKiPWuN18dr2RkhPDkwifPS2R',
  [Network.Mainnet]: '5GDsB8Qm6CAoBi7rmM6TCKMQQUg8CiRzuH9YVyfcrwDKWoqB',
  [Network.Local]: ''
}
export const USDT_ADDRESS = {
  [Network.Testnet]: '5DnXKCPAqTMWWqDoC8NhBo6V8SFh2mqfk3NaRfs6pbQbCZHg',
  [Network.Mainnet]: '5HX57YoV7h51NEKhpfXZAJk8RzLX4Uutp36S23RDMPZ424LY',
  [Network.Local]: ''
}
export const SOL_ADDRESS = {
  [Network.Testnet]: '5Cym4cb86pGxP1NcuXqXMgUMy12Bqeat587Bi5YNQNHKfNQY',
  [Network.Mainnet]: '5F2xiTnahG1tFY3ZHyghh25JsjuCaamRnN7ddQjPEzwvdd3j',
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