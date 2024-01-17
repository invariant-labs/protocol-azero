import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Network } from '../src/network'
import { deployInvariant, deployPSP22, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

describe('tickmap', async () => {
  const feeTier = newFeeTier({ v: 10000000000n }, 1n)
  const ticks = [-58n, -26n, 1n, 3n, 5n]
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
    assert.equal(tickmap.length, 2047)
  })
})
