import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Network } from '../src/network'
import { deployInvariant, deployPSP22, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const startingChunk = 0n
const finishingChunk = 6932n
const chunkOrder = [0n, 1638n, 3276n, 4914n, 6552n, 6932n]

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

describe.only('tickmap', async () => {
  const feeTier = newFeeTier({ v: 10000000000n }, 1n)
  const ticks = [-221818n, -221817n, 0n, 1n, 2n, 221817n, 221818n]
  let poolKey = newPoolKey(
    token0.contract.address.toString(),
    token1.contract.address.toString(),
    feeTier
  )
  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
    token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    await token0.approve(account, invariant.contract.address.toString(), 10000000000n)
    await token1.approve(account, invariant.contract.address.toString(), 10000000000n)
  })

  it('get tickmap', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    await invariant.createPosition(
      account,
      poolKey,
      ticks[2],
      ticks[3],
      { v: 10n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )
    await invariant.createPosition(
      account,
      poolKey,
      ticks[3],
      ticks[4],
      { v: 10n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )
    const tickmap = await invariant.getTickmap(account, poolKey, startingChunk, finishingChunk)
    assert.deepEqual(tickmap, [[3465n, 2017612633061982208n]])
  })
  it('get edge ticks', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    await invariant.createPosition(
      account,
      poolKey,
      ticks[0],
      ticks[1],
      { v: 10n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )
    await invariant.createPosition(
      account,
      poolKey,
      ticks[5],
      ticks[6],
      { v: 10n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )
    const tickmap = await invariant.getTickmap(account, poolKey, startingChunk, finishingChunk)
    assert.deepEqual(tickmap, [
      [0n, 3n],
      [6931n, 6755399441055744n]
    ])
  })
  it('get full width ticks', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    await invariant.createPosition(
      account,
      poolKey,
      ticks[0],
      ticks[6],
      { v: 10n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )
    const tickmap = await invariant.getTickmap(account, poolKey, startingChunk, finishingChunk)
    assert.deepEqual(tickmap, [
      [0n, 1n],
      [6931n, 4503599627370496n]
    ])
  })
  it('get max chunks', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    for (let i = 6n; i < 104838n; i += 64n) {
      await invariant.createPosition(
        account,
        poolKey,
        i,
        i + 1n,
        { v: 10n },
        pool.sqrtPrice,
        pool.sqrtPrice
      )
    }
    const tickmap = await invariant.getTickmap(account, poolKey, startingChunk, finishingChunk)
    assert.equal(tickmap.length, 1638)
  })
  it('get whole tickmap', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    for (let i = 6n; i < 104838n; i += 64n) {
      await invariant.createPosition(
        account,
        poolKey,
        i,
        i + 1n,
        { v: 10n },
        pool.sqrtPrice,
        pool.sqrtPrice
      )
    }

    const tickmap = await Promise.all([
      await invariant.getTickmap(account, poolKey, chunkOrder[0], chunkOrder[1]),
      await invariant.getTickmap(account, poolKey, chunkOrder[1], chunkOrder[2]),
      await invariant.getTickmap(account, poolKey, chunkOrder[2], chunkOrder[3]),
      await invariant.getTickmap(account, poolKey, chunkOrder[3], chunkOrder[4]),
      await invariant.getTickmap(account, poolKey, chunkOrder[4], chunkOrder[5])
    ])

    assert.equal(tickmap.length, 5)
    assert.equal(tickmap[0].length, 0)
    assert.equal(tickmap[1].length, 0)
    assert.equal(tickmap[2].length, 1449)
    assert.equal(tickmap[3].length, 190)
    assert.equal(tickmap[4].length, 0)
  })
})
