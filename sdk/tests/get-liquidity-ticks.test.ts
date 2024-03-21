import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { objectEquals } from '../src/testUtils'
import { initPolkadotApi, integerSafeCast, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

const feeTier = newFeeTier(10000000000n, 1n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

describe('get-liquidity-ticks', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  })

  it('should get liquidity ticks', async function () {
    await invariant.createPosition(account, poolKey, -10n, 10n, 10n, 1000000000000000000000000n, 0n)

    const result = await invariant.getLiquidityTicks(account, poolKey, 0n)
    assert.equal(result.length, 2)

    const lowerTick = await invariant.getTick(account, poolKey, -10n)
    const upperTick = await invariant.getTick(account, poolKey, 10n)

    objectEquals(result[0], lowerTick, [])
    objectEquals(result[1], upperTick, [])
  })

  it('should get liquidity ticks limit', async function () {
    this.timeout(30000)

    for (let i = 1n; i <= 390n; i++) {
      await invariant.createPosition(account, poolKey, -i, i, 10n, 1000000000000000000000000n, 0n)
    }

    const result = await invariant.getLiquidityTicks(account, poolKey, 0n)
    assert.equal(result.length, 780)

    for (let i = -390n; i <= 390n; i++) {
      if (i !== 0n) {
        const tick = await invariant.getTick(account, poolKey, i)

        if (i > 0n) {
          objectEquals(result[integerSafeCast(i) + 390 - 1], tick, [])
        } else {
          objectEquals(result[integerSafeCast(i) + 390], tick, [])
        }
      }
    }
  })

  it('should get liquidity ticks with offset', async () => {
    await invariant.createPosition(account, poolKey, -10n, 10n, 10n, 1000000000000000000000000n, 0n)

    const result1 = await invariant.getLiquidityTicks(account, poolKey, 0n)
    assert.equal(result1.length, 2)

    const result2 = await invariant.getLiquidityTicks(account, poolKey, 1n)
    assert.equal(result2.length, 1)

    objectEquals(result1[1], result2[0], [])
  })

  it('should get liquidity ticks with multiple queries', async function () {
    this.timeout(25000)

    for (let i = 1n; i <= 400n; i++) {
      await invariant.createPosition(account, poolKey, -i, i, 10n, 1000000000000000000000000n, 0n)
    }

    const liquidityTicks = await invariant.getLiquidityTicksAmount(account, poolKey)

    const promises: any[] = []

    for (let i = 0n; i < liquidityTicks; i += 780n) {
      promises.push(invariant.getLiquidityTicks(account, poolKey, i))
    }

    const result = await Promise.all(promises)

    let ticks = 0

    for (let i = 0; i < result.length; i++) {
      ticks += result[i].length
    }

    assert.equal(ticks, 800)
  })
})
