import {
  Tick,
  Tickmap,
  getMaxChunk,
  getMaxTick,
  getMinTick
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { assert, expect } from 'chai'
import { CHUNK_SIZE, MAX_TICKMAP_QUERY_SIZE } from '../src/consts'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { delay, getActiveBitsCount64, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

describe('tickmap test', async () => {
  const feeTier = newFeeTier(10000000000n, 1n)
  const ticks = [-221818n, -221817n, -58n, 5n, 221817n, 221818n]
  let poolKey = newPoolKey(token0Address, token1Address, feeTier)
  beforeEach(async function () {
    this.timeout(200000)

    await api.disconnect()
    await delay(2000)
    await api.connect()

    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000000000n,
      token1Address
    )
  })

  it('get tickmap', async () => {
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)
    await invariant.createPosition(account, poolKey, ticks[2], ticks[3], 10n, pool.sqrtPrice, 0n)

    const tickmap = await invariant.getFullTickmap(poolKey)
    assert.deepEqual(tickmap.bitmap.get(3465n), 9223372036854775809n)

    for (const [chunkIndex, value] of tickmap.bitmap.entries()) {
      if (chunkIndex === 3465n) {
        assert.deepEqual(value, 0b1000000000000000000000000000000000000000000000000000000000000001n)
      } else {
        assert.deepEqual(value, 0n)
      }
    }
  })
  it('get tickmap edge ticks initialized', async () => {
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)
    await invariant.createPosition(account, poolKey, ticks[0], ticks[1], 10n, pool.sqrtPrice, 0n)
    await invariant.createPosition(account, poolKey, ticks[4], ticks[5], 10n, pool.sqrtPrice, 0n)

    const tickmap = await invariant.getFullTickmap(poolKey)

    assert.deepEqual(tickmap.bitmap.get(0n), 0b11n)
    assert.deepEqual(
      tickmap.bitmap.get(getMaxChunk(feeTier.tickSpacing)),
      0b11000000000000000000000000000000000000000000000000000n
    )
  })
  it('get tickmap edge ticks initialized 100 tick spacing', async () => {
    const feeTier100 = newFeeTier(10000000000n, 100n)
    poolKey = newPoolKey(token0Address, token1Address, feeTier100)
    await invariant.addFeeTier(account, feeTier100)
    await invariant.createPool(account, poolKey, 1000000000000000000000000n)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier100)
    await invariant.createPosition(
      account,
      poolKey,
      getMinTick(feeTier100.tickSpacing),
      getMaxTick(feeTier100.tickSpacing),
      100n,
      pool.sqrtPrice,
      0n
    )
    await invariant.createPosition(
      account,
      poolKey,
      getMinTick(feeTier100.tickSpacing) + feeTier100.tickSpacing,
      getMaxTick(feeTier100.tickSpacing) - feeTier100.tickSpacing,
      100n,
      pool.sqrtPrice,
      0n
    )

    const tickmap = await invariant.getFullTickmap(poolKey)

    assert.deepEqual(tickmap.bitmap.get(0n), 0b11n)

    assert.deepEqual(
      tickmap.bitmap.get(getMaxChunk(feeTier100.tickSpacing)),
      0b110000000000000000000n
    )
  })
  it('get tickmap more chunks above', async function () {
    this.timeout(200000)

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    for (let i = 6n; i < 52500n; i += 64n) {
      await invariant.createPosition(account, poolKey, i, i + 1n, 10n, pool.sqrtPrice, 0n)
    }

    const tickmap = await invariant.getFullTickmap(poolKey)

    const initializedChunks = 52500n / 64n
    for (let i = 0n; i < initializedChunks; i++) {
      const current = 3466n + i
      assert.deepEqual(tickmap.bitmap.get(current), 0b11n)
    }
  })
  it('get tickmap more chunks below', async function () {
    this.timeout(200000)

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    // 51328
    for (let i = -52544n; i < 6n; i += 64n) {
      await invariant.createPosition(account, poolKey, i, i + 1n, 10n, pool.sqrtPrice, 0n)
    }

    const tickmap = await invariant.getFullTickmap(poolKey)
    const initializedChunks = 52544n / 64n
    for (let i = 0n; i < initializedChunks; i++) {
      const current = 2644n + i
      assert.deepEqual(
        tickmap.bitmap.get(current),
        0b110000000000000000000000000000000000000000000000000000000000n
      )
    }
  })

  it('get_tickmap query size exceeds max query size', async function () {
    this.timeout(2000000)
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    const mintAmount = 1n << 120n
    await psp22.mint(account, account.address, mintAmount, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), mintAmount, token0Address)
    await psp22.mint(account, account.address, mintAmount, token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), mintAmount, token1Address)

    const liquidityDelta = 10000000n * 10n ** 6n
    const spotSqrtPrice = 1000000000000000000000000n
    const slippageTolerance = 0n

    const indexes: bigint[] = []
    for (let i = -(MAX_TICKMAP_QUERY_SIZE + 1n) * CHUNK_SIZE; i < 0n; i += 2n * CHUNK_SIZE) {
      indexes.push(i)
      indexes.push(i + CHUNK_SIZE)
      await invariant.createPosition(
        account,
        poolKey,
        i,
        i + CHUNK_SIZE,
        liquidityDelta,
        spotSqrtPrice,
        slippageTolerance
      )
    }

    assert(BigInt(indexes.length) > MAX_TICKMAP_QUERY_SIZE, 'test sample insufficient')

    const promises: Promise<Tick>[] = []

    for (const index of indexes) {
      promises.push(invariant.getTick(poolKey, index))
    }
    const ticks = await Promise.all(promises)

    const tickmap: Tickmap = await invariant.getFullTickmap(poolKey)

    let sum = 0n
    for (const [, chunk] of tickmap.bitmap) {
      sum += getActiveBitsCount64(chunk)
    }

    expect(sum).to.equal(BigInt(ticks.length))
    expect(sum).to.equal(BigInt(indexes.length))
  })
})
