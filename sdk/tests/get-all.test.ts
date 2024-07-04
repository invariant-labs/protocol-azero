import { PoolKey, toPercentage } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { assert } from 'chai'
import { describe, it } from 'mocha'
import { SQRT_PRICE_DENOMINATOR } from '../src/consts'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, toPercentage(1n, 2n))
let token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let token2Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

const feeTier = newFeeTier(6000000000n, 10n)

let poolKey = newPoolKey(token0Address, token1Address, feeTier)

describe('get-all', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, toPercentage(1n, 2n))
    token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    token2Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token2Address)
  })

  it('get all pool keys works', async function () {
    this.timeout(300000)

    const feeTiers = Array.from(Array(10).keys()).map(i => newFeeTier(BigInt(i + 1), BigInt(i + 1)))
    const expectedPoolKeys: PoolKey[] = []
    for (const feeTier of feeTiers) {
      await invariant.addFeeTier(account, feeTier)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      expectedPoolKeys.push(poolKey)
      await invariant.createPool(account, poolKey, SQRT_PRICE_DENOMINATOR)
    }

    const poolKeys = await invariant.getAllPoolKeys()
    assert.equal(poolKeys.length, 10)

    poolKeys.map((poolKey, index) => {
      assert.deepEqual(poolKey, expectedPoolKeys[index])
    })
  })

  it('get all pool keys above single query limit works', async function () {
    this.timeout(300000)

    const feeTiers = Array.from(Array(100).keys()).map(i =>
      newFeeTier(BigInt(i + 1), BigInt(i + 1))
    )
    const expectedPoolKeys: PoolKey[] = []
    for (const feeTier of feeTiers) {
      await invariant.addFeeTier(account, feeTier)

      let poolKey = newPoolKey(token0Address, token1Address, feeTier)
      expectedPoolKeys.push(poolKey)
      await invariant.createPool(account, poolKey, SQRT_PRICE_DENOMINATOR)
      poolKey = newPoolKey(token0Address, token2Address, feeTier)
      expectedPoolKeys.push(poolKey)
      await invariant.createPool(account, poolKey, SQRT_PRICE_DENOMINATOR)
      poolKey = newPoolKey(token1Address, token2Address, feeTier)
      expectedPoolKeys.push(poolKey)
      await invariant.createPool(account, poolKey, SQRT_PRICE_DENOMINATOR)
    }

    const poolKeys = await invariant.getAllPoolKeys()
    assert.equal(poolKeys.length, 300)

    poolKeys.map((poolKey, index) => {
      assert.deepEqual(poolKey, expectedPoolKeys[index])
    })
  })

  it('get all positions works', async function () {
    this.timeout(300000)

    await invariant.addFeeTier(account, feeTier)
    await invariant.createPool(
      account,
      newPoolKey(token0Address, token1Address, feeTier),
      SQRT_PRICE_DENOMINATOR
    )
    for (let i = 0; i < 10; i++) {
      await invariant.createPosition(
        account,
        poolKey,
        -10n,
        10n,
        1000000n,
        SQRT_PRICE_DENOMINATOR,
        0n
      )
    }

    const positionsEntries = await invariant.getAllPositions(account.address)
    assert.equal(positionsEntries.length, 10)

    for (const [index, [position, pool, lowerTick, upperTick]] of positionsEntries.entries()) {
      const expectedPosition = await invariant.getPosition(account.address, BigInt(index))
      const expectedPool = await invariant.getPool(
        expectedPosition.poolKey.tokenX,
        expectedPosition.poolKey.tokenY,
        expectedPosition.poolKey.feeTier
      )
      const expectedLowerTick = await invariant.getTick(
        expectedPosition.poolKey,
        expectedPosition.lowerTickIndex
      )
      const expectedUpperTick = await invariant.getTick(
        expectedPosition.poolKey,
        expectedPosition.upperTickIndex
      )

      assert.deepEqual(position, expectedPosition)
      assert.deepEqual(pool, expectedPool)
      assert.deepEqual(lowerTick, expectedLowerTick)
      assert.deepEqual(upperTick, expectedUpperTick)
    }
  })

  it('get all positions above single query limit works', async function () {
    this.timeout(300000)

    await invariant.addFeeTier(account, feeTier)
    await invariant.createPool(
      account,
      newPoolKey(token0Address, token1Address, feeTier),
      SQRT_PRICE_DENOMINATOR
    )
    for (let i = 0; i < 50; i++) {
      await invariant.createPosition(
        account,
        poolKey,
        -10n,
        10n,
        1000000n,
        SQRT_PRICE_DENOMINATOR,
        0n
      )
    }

    const positionsEntries = await invariant.getAllPositions(account.address)
    assert.equal(positionsEntries.length, 50)

    for (const [index, [position, pool, lowerTick, upperTick]] of positionsEntries.entries()) {
      const expectedPosition = await invariant.getPosition(account.address, BigInt(index))
      const expectedPool = await invariant.getPool(
        expectedPosition.poolKey.tokenX,
        expectedPosition.poolKey.tokenY,
        expectedPosition.poolKey.feeTier
      )
      const expectedLowerTick = await invariant.getTick(
        expectedPosition.poolKey,
        expectedPosition.lowerTickIndex
      )
      const expectedUpperTick = await invariant.getTick(
        expectedPosition.poolKey,
        expectedPosition.upperTickIndex
      )

      assert.deepEqual(position, expectedPosition)
      assert.deepEqual(pool, expectedPool)
      assert.deepEqual(lowerTick, expectedLowerTick)
      assert.deepEqual(upperTick, expectedUpperTick)
    }
  })
})
