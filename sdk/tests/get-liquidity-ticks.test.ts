import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { objectEquals } from '../src/testUtils'
import {
  initPolkadotApi,
  integerSafeCast,
  newFeeTier,
  newPoolKey,
  positionToTick
} from '../src/utils'
import { CHUNK_SIZE, LIQUIDITY_TICKS_LIMIT } from '../src/consts'
import { describe, it } from 'mocha'

const network = Network.Local
const api = await initPolkadotApi(network)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

const deployOptions = {
  storageDepositLimit: null,
  refTime: 259058343000,
  proofSize: 1160117000
}

let invariant = await Invariant.deploy(api, network, account, 10000000000n, deployOptions)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, network, deployOptions)

const feeTier = newFeeTier(10000000000n, 1n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

describe('get-liquidity-ticks', async () => {
  beforeEach(async function () {
    this.timeout(20000)
    invariant = await Invariant.deploy(api, network, account, 10000000000n, deployOptions)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token1Address)
  })

  it('should get liquidity ticks limit', async function () {
    this.timeout(1200000)
    const tickLimit = integerSafeCast(LIQUIDITY_TICKS_LIMIT)

    for (let i = 1n; i <= 390n; i++) {
      await invariant.createPosition(account, poolKey, -i, i, 10n, 1000000000000000000000000n, 0n)
    }
    const tickmap = await invariant.getFullTickmap(poolKey)

    const tickIndexes: bigint[] = []

    for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
      for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
        const checkedBit = chunk & (1n << bit)
        if (checkedBit) {
          const tickIndex = positionToTick(chunkIndex, bit, poolKey.feeTier.tickSpacing)
          tickIndexes.push(tickIndex)
        }
      }
    }
    const singleQueryLiquidityTicks = await invariant.getLiquidityTicks(poolKey, tickIndexes)
    assert.equal(singleQueryLiquidityTicks.length, tickLimit)

    for (let i = -390n; i <= 390n; i++) {
      if (i !== 0n) {
        const tick = await invariant.getTick(poolKey, i)

        if (i > 0n) {
          objectEquals(singleQueryLiquidityTicks[integerSafeCast(i) + 390 - 1], tick, [
            'index',
            'liquidity',
            'sign'
          ])
        } else {
          objectEquals(singleQueryLiquidityTicks[integerSafeCast(i) + 390], tick, [
            'index',
            'liquidity',
            'sign'
          ])
        }
      }
    }

    const allLiquidityTicks = await invariant.getAllLiquidityTicks(poolKey, tickmap)
    assert.equal(allLiquidityTicks.length, tickLimit)
    for (let i = 0; i < allLiquidityTicks.length; i++) {
      assert.deepEqual(allLiquidityTicks[i], singleQueryLiquidityTicks[i])
    }
  })

  it('should get liquidity ticks with multiple queries', async function () {
    this.timeout(1200000)

    for (let i = 1n; i <= 400n; i++) {
      await invariant.createPosition(account, poolKey, -i, i, 10n, 1000000000000000000000000n, 0n)
    }

    const tickmap = await invariant.getFullTickmap(poolKey)

    const tickIndexes: bigint[] = []
    for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
      for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
        const checkedBit = chunk & (1n << bit)
        if (checkedBit) {
          const tickIndex = positionToTick(chunkIndex, bit, poolKey.feeTier.tickSpacing)
          tickIndexes.push(tickIndex)
        }
      }
    }
    assert.equal(tickIndexes.length, 800)

    const tickLimit = integerSafeCast(LIQUIDITY_TICKS_LIMIT)

    const firstQuery = await invariant.getLiquidityTicks(
      poolKey,
      tickIndexes.slice(0, tickLimit)
    )
    const secondQuery = await invariant.getLiquidityTicks(
      poolKey,
      tickIndexes.slice(tickLimit, 800)
    )

    assert.equal(firstQuery.length, tickLimit)
    assert.equal(secondQuery.length, 20)

    const fullQuery = firstQuery.concat(secondQuery)
    assert.equal(fullQuery.length, 800)

    for (let i = 0; i < 800; i++) {
      assert(fullQuery[i].index === tickIndexes[i])
    }

    const liquidityTicks = await invariant.getAllLiquidityTicks(poolKey, tickmap)
    assert.equal(liquidityTicks.length, 800)
    for (let i = 0; i < liquidityTicks.length; i++) {
      assert.deepEqual(liquidityTicks[i], fullQuery[i])
    }
  })
})
