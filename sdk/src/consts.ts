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
  [Network.Testnet]: '5EmtoXQqLPw5Qu43ZkFRzjRLUmkzDV9cKTZm8ADmnvS78BpV',
  [Network.Mainnet]: '5FVYdhN1rCSQeVdYch84DXyanHEtVyeSCE482oGXb6YG7AGk',
  [Network.Local]: ''
}
export const BTC_ADDRESS = {
  [Network.Testnet]: '5Cr6XesdYm2eC9Fxtm5uLP1c8EzKSyzWFgtB56bhZaLAchWq',
  [Network.Mainnet]: '5Gbp6cdVtWtQpRrKvPkZV5PTRKPPytc2D5tDVaC5Bg5VYARx',
  [Network.Local]: ''
}
export const ETH_ADDRESS = {
  [Network.Testnet]: '5FFdg7r8Xckkyhtj5NNRqXvBsToZYvciRsQQ2aEcojUGqEqp',
  [Network.Mainnet]: '5D1GnFQZiQVdFgBRRuFbAgxis9GscFzUU9f5PQjuzEcZdsgU',
  [Network.Local]: ''
}
export const USDC_ADDRESS = {
  [Network.Testnet]: '5CqkYZrAg89tAKFiNEFDVCNAccQ6MC5p3jBa8u5ECD8pcHgg',
  [Network.Mainnet]: '5DJz7bTVts3YVrGhHFVawiQbwzfiPZxrRY5S9vnfFDYctC9w',
  [Network.Local]: ''
}
export const USDT_ADDRESS = {
  [Network.Testnet]: '5CTMVdWxf8UeCojQTcvka4qAEXwLHFnpJ6JXxwASWPzn9i3t',
  [Network.Mainnet]: '5GE5H4dFDYfb8NV64pvzoxd9n1pJTVpoRAtfymsVBdVUosW3',
  [Network.Local]: ''
}
export const SOL_ADDRESS = {
  [Network.Testnet]: '5GVm1gVJNWtYyi6iFE7XsKC1nKKLBQHw78g7Hvgxzrj3QWta',
  [Network.Mainnet]: '5HBhXtmAGnsFYge32TT71cnVcCbky8NHqunrWTdixhSfbjtN',
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

export const RPC = {
  [Network.Testnet]: 'wss://ws.test.azero.dev',
  [Network.Mainnet]: 'wss://ws.azero.dev',
  [Network.Local]: undefined
}
