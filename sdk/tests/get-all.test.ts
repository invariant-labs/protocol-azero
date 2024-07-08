import { PoolKey, toPercentage } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { assert } from 'chai'
import { describe, it } from 'mocha'
import { POSITIONS_ENTRIES_LIMIT, SQRT_PRICE_DENOMINATOR } from '../src/consts'

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
  beforeEach(async function () {
    this.timeout(10000)

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
    this.timeout(30000)

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
    this.timeout(120000)

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
    this.timeout(30000)

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
        -BigInt((i + 1) * 10),
        BigInt((i + 1) * 10),
        1000000n,
        SQRT_PRICE_DENOMINATOR,
        0n
      )
    }

    const pages = await invariant.getAllPositions(account.address)
    assert.equal(pages.map(page => page.entries).flat(1).length, 10)

    for (const { index, entries } of pages) {
      for (const [positionIndex, [position, pool]] of entries.entries()) {
        const expectedPosition = await invariant.getPosition(
          account.address,
          BigInt((index - 1) * Number(POSITIONS_ENTRIES_LIMIT) + positionIndex)
        )
        const expectedPool = await invariant.getPool(
          expectedPosition.poolKey.tokenX,
          expectedPosition.poolKey.tokenY,
          expectedPosition.poolKey.feeTier
        )

        assert.deepEqual(position, expectedPosition)
        assert.deepEqual(pool, expectedPool)
      }
    }
  })

  it('get all positions above single query limit works', async function () {
    this.timeout(30000)

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
        -BigInt((i + 1) * 10),
        BigInt((i + 1) * 10),
        1000000n,
        SQRT_PRICE_DENOMINATOR,
        0n
      )
    }

    const pages = await invariant.getAllPositions(account.address)
    assert.equal(pages.map(page => page.entries).flat(1).length, 50)

    for (const { index, entries } of pages) {
      for (const [positionIndex, [position, pool]] of entries.entries()) {
        const expectedPosition = await invariant.getPosition(
          account.address,
          BigInt((index - 1) * Number(POSITIONS_ENTRIES_LIMIT) + positionIndex)
        )
        const expectedPool = await invariant.getPool(
          expectedPosition.poolKey.tokenX,
          expectedPosition.poolKey.tokenY,
          expectedPosition.poolKey.feeTier
        )

        assert.deepEqual(position, expectedPosition)
        assert.deepEqual(pool, expectedPool)
      }
    }
  })

  it('get all positions with positions count', async function () {
    this.timeout(300000)

    await invariant.addFeeTier(account, feeTier)
    await invariant.createPool(
      account,
      newPoolKey(token0Address, token1Address, feeTier),
      SQRT_PRICE_DENOMINATOR
    )
    for (let i = 0; i < 60; i++) {
      await invariant.createPosition(
        account,
        poolKey,
        -BigInt((i + 1) * 10),
        BigInt((i + 1) * 10),
        1000000n,
        SQRT_PRICE_DENOMINATOR,
        0n
      )
    }

    const pages = await invariant.getAllPositions(account.address, 50n)
    assert.equal(pages.map(page => page.entries).flat(1).length, 51)

    for (const { index, entries } of pages) {
      for (const [positionIndex, [position, pool]] of entries.entries()) {
        const expectedPosition = await invariant.getPosition(
          account.address,
          BigInt((index - 1) * Number(POSITIONS_ENTRIES_LIMIT) + positionIndex)
        )
        const expectedPool = await invariant.getPool(
          expectedPosition.poolKey.tokenX,
          expectedPosition.poolKey.tokenY,
          expectedPosition.poolKey.feeTier
        )

        assert.deepEqual(position, expectedPosition)
        assert.deepEqual(pool, expectedPool)
      }
    }
  })

  it('get all positions with skip pages', async function () {
    this.timeout(300000)

    await invariant.addFeeTier(account, feeTier)
    await invariant.createPool(
      account,
      newPoolKey(token0Address, token1Address, feeTier),
      SQRT_PRICE_DENOMINATOR
    )
    for (let i = 0; i < 200; i++) {
      await invariant.createPosition(
        account,
        poolKey,
        -BigInt((i + 1) * 10),
        BigInt((i + 1) * 10),
        1000000n,
        SQRT_PRICE_DENOMINATOR,
        0n
      )
    }

    const pages = await invariant.getAllPositions(account.address, undefined, [2, 4])
    assert.equal(pages.map(page => page.entries).flat(1).length, 102)

    for (const { index, entries } of pages) {
      for (const [positionIndex, [position, pool]] of entries.entries()) {
        const expectedPosition = await invariant.getPosition(
          account.address,
          BigInt((index - 1) * Number(POSITIONS_ENTRIES_LIMIT) + positionIndex)
        )
        const expectedPool = await invariant.getPool(
          expectedPosition.poolKey.tokenX,
          expectedPosition.poolKey.tokenY,
          expectedPosition.poolKey.feeTier
        )
        assert.deepEqual(position, expectedPosition)
        assert.deepEqual(pool, expectedPool)
      }
    }

    it('get all positions with positions count and skip pages', async function () {
      this.timeout(300000)

      await invariant.addFeeTier(account, feeTier)
      await invariant.createPool(
        account,
        newPoolKey(token0Address, token1Address, feeTier),
        SQRT_PRICE_DENOMINATOR
      )
      for (let i = 0; i < 160n; i++) {
        await invariant.createPosition(
          account,
          poolKey,
          -BigInt((i + 1) * 10),
          BigInt((i + 1) * 10),
          1000000n,
          SQRT_PRICE_DENOMINATOR,
          0n
        )
      }

      const pages = await invariant.getAllPositions(account.address, 140n, [1, 2])
      assert.equal(pages.map(page => page.entries).flat(1).length, 38)

      for (const { index, entries } of pages) {
        for (const [positionIndex, [position, pool]] of entries.entries()) {
          const expectedPosition = await invariant.getPosition(
            account.address,
            BigInt((index - 1) * Number(POSITIONS_ENTRIES_LIMIT) + positionIndex)
          )
          const expectedPool = await invariant.getPool(
            expectedPosition.poolKey.tokenX,
            expectedPosition.poolKey.tokenY,
            expectedPosition.poolKey.feeTier
          )

          assert.deepEqual(position, expectedPosition)
          assert.deepEqual(pool, expectedPool)
        }
      }
    })
  })
})
