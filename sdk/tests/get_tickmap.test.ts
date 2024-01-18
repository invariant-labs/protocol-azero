import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Network } from '../src/network'
import { assertThrowsAsync } from '../src/testUtils'
import { deployInvariant, deployPSP22, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

describe.only('tickmap', async () => {
  const feeTier = newFeeTier({ v: 10000000000n }, 1n)
  const ticks = [-221818n, -221817n, -58n, 5n, 221817n, 221818n]
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

    const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
    assert.deepEqual(tickmap[0], [3465n, 9223372036854775809n])
    assert.equal(tickmap.length, 1)
  })
  it('get tickmap edge ticks initialized', async () => {
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
      ticks[4],
      ticks[5],
      { v: 10n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )

    const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
    assert.deepEqual(tickmap[0], [0n, 3n])
    assert.deepEqual(tickmap[1], [6931n, 6755399441055744n])
  })
  it('get tickmap more chunks above', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    for (let i = 6n; i < 10048n; i += 64n) {
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

    const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)

    for (let i = 0n; i < tickmap.length; i++) {
      const current = 3466n + i
      assert.deepEqual(tickmap[Number(i)], [current, 3n])
    }
  })
  it('get tickmap more chunks below', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    for (let i = -10048n; i < 6; i += 64n) {
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

    const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)

    for (let i = 0n; i < tickmap.length; i++) {
      const current = 3308n + i
      assert.deepEqual(tickmap[Number(i)], [current, 864691128455135232n])
    }
  })
  it('get tickmap max chunks returned', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    for (let i = 0n; i < 104832n; i += 64n) {
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

    const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
    assert.equal(tickmap.length, 1638)
  })
  it('get tickmap max chunks + 1 returned', async () => {
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    for (let i = 0n; i < 104896n; i += 64n) {
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

    assertThrowsAsync(invariant.getTickmap(account, poolKey, pool.currentTickIndex))
  })
})
