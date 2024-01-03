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

  it('create pool', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 5)
    await invariant.addFeeTier(account, feeTier)
    let addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = { v: 1000000000000000000n }
    const initTick = 1n

    const createPoolResult = await invariant.createPool(
      account,
      token0.address,
      token1.address,
      feeTier,
      initSqrtPrice,
      initTick
    )

    // console.log(createPoolResult)

    const result = await invariant.getPool(account, token0.address, token1.address, feeTier)
    console.log(result)

    // const pools = await invariant.getPools(account)
    // console.log(pools)
  })
})
