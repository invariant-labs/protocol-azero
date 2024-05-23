import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { PositionTick } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
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
const psp22 = await PSP22.load(api, Network.Local)

const feeTier = newFeeTier(10000000000n, 1n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

describe('get-position-ticks', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token1Address)
  })

  it('should get position ticks', async () => {
    await invariant.createPosition(account, poolKey, -10n, 10n, 10n, 1000000000000000000000000n, 0n)

    const result = await invariant.getPositionTicks(account.address, 0n)
    assert.equal(result.length, 2)

    const lowerTick = await invariant.getTick(poolKey, -10n)
    const upperTick = await invariant.getTick(poolKey, 10n)

    objectEquals(result[0], lowerTick, [])
    objectEquals(result[1], upperTick, [])
  })

  it('should get position ticks limit', async function () {
    this.timeout(20000)

    for (let i = 1n; i <= 186n; i++) {
      await invariant.createPosition(account, poolKey, -i, i, 10n, 1000000000000000000000000n, 0n)
    }

    const result = await invariant.getPositionTicks(account.address, 0n)
    assert.equal(result.length, 372)

    for (let i = 1n; i <= 186n; i++) {
      const lowerTick = await invariant.getTick(poolKey, -i)
      const upperTick = await invariant.getTick(poolKey, i)

      objectEquals(result[integerSafeCast(i) * 2 - 2], lowerTick, [])
      objectEquals(result[integerSafeCast(i) * 2 - 1], upperTick, [])
    }
  })

  it('should get position ticks with offset', async () => {
    await invariant.createPosition(account, poolKey, -10n, 10n, 10n, 1000000000000000000000000n, 0n)

    await invariant.createPosition(account, poolKey, -20n, 20n, 10n, 1000000000000000000000000n, 0n)

    const result1 = await invariant.getPositionTicks(account.address, 0n)
    assert.equal(result1.length, 4)

    const result2 = await invariant.getPositionTicks(account.address, 1n)
    assert.equal(result2.length, 2)

    objectEquals(result1[2], result2[0], [])
    objectEquals(result1[3], result2[1], [])
  })

  it('should get position ticks with multiple queries', async function () {
    this.timeout(25000)

    for (let i = 1n; i <= 400n; i++) {
      await invariant.createPosition(account, poolKey, -i, i, 10n, 1000000000000000000000000n, 0n)
    }

    const positionAmount = await invariant.getUserPositionAmount(account.address)

    const promises: Promise<PositionTick[]>[] = []

    for (let i = 0n; i < positionAmount; i += 186n) {
      promises.push(invariant.getPositionTicks(account.address, i))
    }

    const result = await Promise.all(promises)

    let ticks = 0

    for (let i = 0; i < result.length; i++) {
      ticks += result[i].length
    }

    assert.equal(ticks, 800)
  })
})
