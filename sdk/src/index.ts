import { Keyring } from '@polkadot/api'
import wasm, {
  Tickmap,
  getChunkSize,
  getMaxChunk,
  getMaxTick
} from '../src/wasm/pkg/invariant_a0_wasm.js'
import {
  simulateInvariantSwap,
  Tick,
  FeeTier,
  PoolKey,
  toPercentage,
  getMinSqrtPrice,
  getMaxSqrtPrice,
  Pool,
} from '../src/wasm/pkg/invariant_a0_wasm.js'
import { newFeeTier, newPoolKey } from './utils.js'

const main = async () => {

  let tick: Tick = {
    index: 0n,
    liquidityGross: 1n,
    feeGrowthOutsideX: 100n,
    feeGrowthOutsideY: 100n,
    sign: false,
    liquidityChange: 1n,
    sqrtPrice: 1n,
    secondsOutside: 100n
  }
  let pool: Pool = {
    liquidity: 100n,
    sqrtPrice: 1000000000000000000000000n,
    currentTickIndex: -0n,
    feeGrowthGlobalX: 100n,
    feeGrowthGlobalY: 100n,
    startTimestamp: 100n,
    feeProtocolTokenX: 100n,
    feeProtocolTokenY: 100n,
    lastTimestamp: 100n,
    feeReceiver: 'reciever'
  }
  let tick_spacing = 100n
  let feeTier = newFeeTier(100n, tick_spacing)
  let poolKey = newPoolKey('tokenX', 'tokenY', feeTier)

  let tickmap = { bitmap: new Map<bigint, bigint>([[34n, 4398046511104n]]) }
  
  try {
    simulateInvariantSwap(
      tickmap,
      100n,
      feeTier,
      pool,
      [tick],
      true,
      1000000n,
      true,
      getMinSqrtPrice(tick_spacing)
    )
  } catch(e) {
    console.log(e)
  }
}
await main()
