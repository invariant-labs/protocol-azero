import { PoolKey } from '@invariant-labs/a0-sdk-wasm'
import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant: Invariant
let token0Address: string
let token1Address: string
let poolKey0: PoolKey
let poolKey1: PoolKey

const psp22 = await PSP22.load(api, Network.Local)

const feeTier10ts = newFeeTier(6000000000n, 10n)
const feeTier20ts = newFeeTier(6000000000n, 20n)

describe('query-on-pair', async () => {
  before(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey0 = newPoolKey(token0Address, token1Address, feeTier10ts)
    poolKey1 = newPoolKey(token0Address, token1Address, feeTier20ts)

    await invariant.addFeeTier(account, feeTier10ts)
    await invariant.addFeeTier(account, feeTier20ts)

    await invariant.createPool(account, poolKey0, 1000000000000000000000000n)
    await invariant.createPool(account, poolKey1, 2000000000000000000000000n)
  })
  it('query all pool keys for pair', async () => {
    const poolKeys = await invariant.getAllPoolKeysForPair(token0Address, token1Address)
    const hasAll = poolKeys.some(
      poolKey =>
        (poolKey.tokenX === poolKey0.tokenX && poolKey.tokenY === poolKey0.tokenY) ||
        (poolKey.tokenX === poolKey1.tokenX && poolKey.tokenY === poolKey1.tokenY)
    )
    assert.isTrue(hasAll)
  })
  it('query all pools for pair', async () => {
    const pools = await invariant.getAllPoolsForPair(token0Address, token1Address)
    const expectedPool0 = await invariant.getPool(
      poolKey0.tokenX,
      poolKey0.tokenY,
      poolKey0.feeTier
    )
    const expectedPool1 = await invariant.getPool(
      poolKey1.tokenX,
      poolKey1.tokenY,
      poolKey1.feeTier
    )
    const hasAll = pools.some(
      pool =>
        pool.sqrtPrice === expectedPool0.sqrtPrice || pool.sqrtPrice === expectedPool1.sqrtPrice
    )
    assert.isTrue(hasAll)
  })
  it('check pagination', async function () {
    this.timeout(30000)

    const poolsToCreate = 250

    for (let i = 0; i < poolsToCreate; i++) {
      const token0 = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
      const token1 = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

      const poolKey = newPoolKey(token0, token1, feeTier10ts)
      await invariant.createPool(account, poolKey, 1000000000000000000000000n)
    }

    const poolKeys = await invariant.getAllPoolKeysForPair(token0Address, token1Address)
    const hasAll = poolKeys.some(
      poolKey =>
        (poolKey.tokenX === poolKey0.tokenX && poolKey.tokenY === poolKey0.tokenY) ||
        (poolKey.tokenX === poolKey1.tokenX && poolKey.tokenY === poolKey1.tokenY)
    )
    assert.isTrue(hasAll)
  })
})
