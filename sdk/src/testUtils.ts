import {
  getMinSqrtPrice,
  InvariantError,
  PoolKey,
  simulateInvariantSwap
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { assert } from 'chai'
import { InvariantTx } from './schema.js'
import { PSP22 } from './psp22.js'
import { ApiPromise } from '@polkadot/api'
import { Network } from './network.js'
import { Invariant } from './invariant.js'
import { IKeyringPair } from '@polkadot/types/types'

export const objectEquals = (
  object: { [key: string]: any },
  expectedObject: { [key: string]: any },
  keys: string[]
) => {
  for (const key in object) {
    if (!keys.includes(key)) {
      assert.deepEqual(object[key], expectedObject[key], `Key missing: ${key}`)
    }
  }
}

export const assertThrowsAsync = async (fn: Promise<any>, word?: InvariantError | InvariantTx) => {
  try {
    await fn
  } catch (e: any) {
    if (word) {
      const err = e.toString()
      console.log(err)
      const regex = new RegExp(`${word}$`)
      if (!regex.test(err)) {
        console.log(err)
        throw new Error('Invalid Error message')
      }
    }
    return
  }
  throw new Error('Function did not throw error')
}

export const getMaxCrossesLimit = async (
  api: ApiPromise,
  network: Network,
  invariant: Invariant,
  poolKey: PoolKey,
  swapper: IKeyringPair,
  initAmount: bigint = 500n,
  tickSampleCount: bigint = 50000n,
) => {
  const psp22 = await PSP22.load(api, network)

  const mintAmount = 1n << 110n
  const tickmap = await invariant.getFullTickmap(poolKey)
  const liquidityTicks = await invariant.getAllLiquidityTicks(poolKey, tickmap)
  const poolBeforeSwap = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)
  await psp22.mint(swapper, mintAmount, poolKey.tokenX)
  await psp22.approve(swapper, invariant.contract.address.toString(), mintAmount, poolKey.tokenX)
  const invBalance = await psp22.balanceOf(invariant.contract.address.toString(), poolKey.tokenY)

  const tickCountToAmountOut = new Array<bigint>()

  // determine tick crossing points
  for (let i = initAmount; i < invBalance; i += invBalance / tickSampleCount + 1n) {
    const sim = simulateInvariantSwap(
      tickmap,
      poolKey.feeTier,
      poolBeforeSwap,
      liquidityTicks,
      true,
      i,
      false,
      getMinSqrtPrice(1n)
    )

    if (sim.globalInsufficientLiquidity || sim.maxSwapStepsReached) {
      console.log(sim)
      break
    }
    tickCountToAmountOut[sim.crossedTicks.length] = sim.amountOut
  }

  tickCountToAmountOut.sort((a, b) => {
    return Number(b - a)
  })

  console.log(invBalance)
  console.log(tickCountToAmountOut)

  let prev;
  for (let i = 0; i < tickCountToAmountOut.length; i++) {
    const tickCountAndBalance = tickCountToAmountOut[i]
    try {
      if (tickCountAndBalance) {
        if (prev && prev != i - 1) {
          console.warn(`index skipped between ${prev} and ${i}, tick sample count may be too low`)
        }

        console.log('swapping', tickCountAndBalance, i)
        const txResult = await invariant.swap(
          swapper,
          poolKey,
          true,
          tickCountAndBalance,
          false,
          getMinSqrtPrice(1n)
        )
        console.log("Final Swap", txResult.events)
        return txResult.events
      }
    } catch (e) {
      console.error('Failed to swap', tickCountAndBalance, i)
    } finally {
      prev = i
    }
    
    throw new Error("Failed to find a swap that wouldn't panic")
  }
}
