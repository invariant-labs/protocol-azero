import { FeeTier } from '@invariant-labs/a0-sdk-wasm'
import { calculateFeeTierWithLinearRatio, getConcentrationArray, integerSafeCast } from './utils.js'

export const FEE_TIERS: FeeTier[] = [
  calculateFeeTierWithLinearRatio(1n),
  calculateFeeTierWithLinearRatio(2n),
  calculateFeeTierWithLinearRatio(5n),
  calculateFeeTierWithLinearRatio(10n),
  calculateFeeTierWithLinearRatio(30n),
  calculateFeeTierWithLinearRatio(100n)
]

export const CONCENTRATION_ARRAY: number[][] = FEE_TIERS.map(tier =>
  getConcentrationArray(integerSafeCast(tier.tickSpacing), 2, 0).sort((a, b) => a - b)
)
