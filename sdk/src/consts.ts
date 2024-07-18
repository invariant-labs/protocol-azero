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
  getMaxTickCross,
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
  getTokenAmountDenominator,
  getTokenAmountScale
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'

export const MAX_REF_TIME = 259058343000
export const DEFAULT_REF_TIME = 1250000000000
export const DEFAULT_PROOF_SIZE = 1250000000000
export const CONCENTRATION_FACTOR = 1.00001526069123

export const TESTNET = 'alephzero-testnet'
export const MAINNET = 'alephzero-mainnet'
export const DEFAULT_LOCAL = 'ws://127.0.0.1:9944'

export const TESTNET_WAZERO_ADDRESS = '5EFDb7mKbougLtr5dnwd5KDfZ3wK55JPGPLiryKq4uRMPR46'

export const TESTNET_INVARIANT_ADDRESS = '5FU7vwthg7LDANp2Ljmw4gYdBY8rAF2UeS3rtM4sFZfnFNXv'
export const TESTNET_BTC_ADDRESS = '5FsnMq7UCvkaniSKiLdas2ujqRfKNiUehE21pQ4eCtZ4Hdgm'
export const TESTNET_ETH_ADDRESS = '5E3MV1sZQSav3BCf9aucDnZ79vRLWWE2eZbuy5dbNbj62qwN'
export const TESTNET_USDC_ADDRESS = '5Duwky8RAzWFKb8GNYcejjmxAAU2EkxPgeWCxMtvcsPdKtHm'
export const TESTNET_USDT_ADDRESS = '5FR9i4PwD9T3fokcLocyzWLCfUs7CstGRnwUSVovMDhnDxrz'
export const TESTNET_SOL_ADDRESS = '5CRMY8uap4JAQPcoJiaM4bHnuHjZLxRWRzYWh118BNiMsPW5'

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
export const MAX_TICK_CROSS = getMaxTickCross()
export const LIQUIDITY_TICKS_LIMIT = getLiquidityTicksLimit()
export const MAX_POOL_KEYS_RETURNED = getMaxPoolKeysReturned()
export const MAX_POOL_PAIRS_RETURNED = getMaxPoolPairsReturned()
export const POSITIONS_ENTRIES_LIMIT = getPositionsEntriesLimit()
