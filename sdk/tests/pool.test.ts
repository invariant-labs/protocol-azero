import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { SqrtPrice, newFeeTier } from 'math/math.js'
import { Network } from '../src/network'
import { deployInvariant, deployPSP22, initPolkadotApi } from '../src/utils'

describe('invariant', async () => {
  const api = await initPolkadotApi(Network.Local)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = await keyring.addFromUri('//Alice')

  let invariant = await deployInvariant(api, account, { v: 10000000000n })
  let token0 = await deployPSP22(api, account, 1000n)
  let token1 = await deployPSP22(api, account, 1000n)

  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n })
    token0 = await deployPSP22(api, account, 1000n)
    token1 = await deployPSP22(api, account, 1000n)
  })

  it.only('create pool', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 1)
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = { v: 1000000000000000000000000n }
    const initTick = 0n

    await invariant.createPool(
      account,
      token0.address,
      token1.address,
      feeTier,
      initSqrtPrice,
      initTick
    )

    await invariant.getPool(account, token0.address, token1.address, feeTier)

    const pools = await invariant.getPools(account)
    console.log('POOLS = ', pools)
    assert.deepEqual(pools.length, 1)
  })
})
