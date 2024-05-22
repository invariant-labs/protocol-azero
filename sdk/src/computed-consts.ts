import { FeeTier } from '@invariant-labs/a0-sdk-wasm'
import { calculateFeeTierWithLinearRatio } from './utils.js'

export const FEE_TIERS: FeeTier[] = [
  calculateFeeTierWithLinearRatio(1n),
  calculateFeeTierWithLinearRatio(2n),
  calculateFeeTierWithLinearRatio(5n),
  calculateFeeTierWithLinearRatio(10n),
  calculateFeeTierWithLinearRatio(30n),
  calculateFeeTierWithLinearRatio(100n)
]
