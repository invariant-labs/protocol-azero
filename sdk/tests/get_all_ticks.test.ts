import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Network } from '../src/network'
import { deployInvariant, deployPSP22, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 10000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 10000000000n, 'Coin', 'COIN', 0n, Network.Local)

const feeTier = newFeeTier({ v: 10000000000n }, 1n)
let poolKey = newPoolKey(
  token0.contract.address.toString(),
  token1.contract.address.toString(),
  feeTier
)

describe('get all ticks', async () => {
  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
    token0 = await deployPSP22(api, account, 10000000000n, 'Coin', 'COIN', 0n, Network.Local)
    token1 = await deployPSP22(api, account, 10000000000n, 'Coin', 'COIN', 0n, Network.Local)

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

  it('should get all ticks', async () => {
    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      { v: 10n },
      { v: 1000000000000000000000000n },
      { v: 1000000000000000000000000n }
    )

    const ticks = await invariant.getAllTicks(account, poolKey)

    const tick1 = await invariant.getTick(account, poolKey, -10n)
    const tick2 = await invariant.getTick(account, poolKey, 10n)

    assert.deepEqual(ticks, [tick1, tick2])
  })

  it('should get all ticks limit', async () => {
    for (let i = 1n; i <= 88n; i++) {
      await invariant.createPosition(
        account,
        poolKey,
        -i,
        i,
        { v: 10n },
        { v: 1000000000000000000000000n },
        { v: 1000000000000000000000000n }
      )
    }

    const ticks = await invariant.getAllTicks(account, poolKey)

    assert.equal(ticks.length, 176)
  })
})
