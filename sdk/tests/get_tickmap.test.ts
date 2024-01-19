import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { assertThrowsAsync } from '../src/testUtils'
import { initPolkadotApi, integerSafeCast, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, { v: 10000000000n })
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

describe.only('tickmap', async () => {
  const feeTier = newFeeTier({ v: 10000000000n }, 1n)
  const ticks = [-221818n, -221817n, -58n, 5n, 221817n, 221818n]
  let poolKey = newPoolKey(token0Address, token1Address, feeTier)
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, { v: 10000000000n })
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0Address,
      token1Address,
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  })

  it('get tickmap', async () => {
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
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
    assert.deepEqual(tickmap[3465], 9223372036854775809n)
    for (const [chunkIndex, value] of tickmap.entries()) {
      if (chunkIndex === 3465) {
        assert.deepEqual(value, 9223372036854775809n)
      } else {
        assert.deepEqual(value, 0n)
      }
    }
  })
  it('get tickmap edge ticks initialized', async () => {
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
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
    assert.deepEqual(tickmap[0], 3n)
    assert.deepEqual(tickmap[6931], 6755399441055744n)
  })
  it('get tickmap more chunks above', async () => {
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

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

    const initializedChunks = 10048n / 64n
    for (let i = 0n; i < initializedChunks; i++) {
      const current = 3466n + i
      assert.deepEqual(tickmap[integerSafeCast(current)], 3n)
    }
  })
  it('get tickmap more chunks below', async () => {
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

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
    const initializedChunks = 10048n / 64n
    for (let i = 0n; i < initializedChunks; i++) {
      const current = 3308n + i
      assert.deepEqual(tickmap[integerSafeCast(current)], 864691128455135232n)
    }
  })
  it('get tickmap max chunks returned', async () => {
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

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

    await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
  })
  it('get tickmap max chunks + 1 returned', async () => {
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

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
