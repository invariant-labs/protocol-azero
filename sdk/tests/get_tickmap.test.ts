import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { newFeeTier } from 'math/math.js'
import { Network } from '../src/network'
import { _newPoolKey, deployInvariant, deployPSP22, initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

describe.only('tickmap', async () => {
  const feeTier = newFeeTier({ v: 10000000000n }, 1)
  const ticks = [-221818n, -221817n, 0n, 1n, 2n, 221817n, 221818n]
  let poolKey = _newPoolKey(
    token0.contract.address.toString(),
    token1.contract.address.toString(),
    feeTier
  )
  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
    token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    poolKey = _newPoolKey(
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
    const tickmap = await invariant.getTickmap(account, poolKey)
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
    const tickmap = await invariant.getTickmap(account, poolKey)
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
    const tickmap = await invariant.getTickmap(account, poolKey)
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
    const tickmap = await invariant.getTickmap(account, poolKey)
    assert.equal(tickmap.length, 1638)
  })
})
