import { Pool } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { describe, it } from 'mocha'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { delay, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Testnet)

const lowerTickIndex = -20n
const upperTickIndex = 10n
const feeTier = newFeeTier(6000000000n, 10n)

let poolKey = newPoolKey(token0Address, token1Address, feeTier)
let pool: Pool

describe('update-position-seconds-per-liquidity', async () => {
  beforeEach(async function () {
    this.timeout(40000)
    invariant = await Invariant.deploy(api, Network.Testnet, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token1Address)

    pool = await invariant.getPool(token0Address, token1Address, feeTier)
  })

  it('position inside', async function () {
    this.timeout(30000)
    await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      1000000000000n,
      pool.sqrtPrice,
      0n
    )

    const positionIndex = 0n

    const poolBefore = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)
    const positionBefore = await invariant.getPosition(account.address, positionIndex)
    assert.equal(poolBefore.secondsPerLiquidityGlobal, 0n)
    assert.equal(positionBefore.secondsPerLiquidityInside, 0n)
    await delay(5000)
    await invariant.updatePositionSecondsPerLiquidity(account, 0n)

    const poolAfter = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)
    const positionAfter = await invariant.getPosition(account.address, positionIndex)

    assert.equal(poolAfter.secondsPerLiquidityGlobal, positionAfter.secondsPerLiquidityInside)
  })

  it('position outside', async function () {
    this.timeout(30000)
    const poolBefore = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)

    const upperTickIndex = poolBefore.currentTickIndex - poolKey.feeTier.tickSpacing
    await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      1000000000000n,
      pool.sqrtPrice,
      0n
    )

    const positionIndex = 0n
    const positionBefore = await invariant.getPosition(account.address, positionIndex)
    assert.equal(poolBefore.secondsPerLiquidityGlobal, 0n)
    assert.equal(positionBefore.secondsPerLiquidityInside, 0n)

    await delay(5000)
    await invariant.updatePositionSecondsPerLiquidity(account, positionIndex)

    const poolAfter = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)
    const lowerTickAfter = await invariant.getTick(poolKey, lowerTickIndex)
    const upperTickAfter = await invariant.getTick(poolKey, upperTickIndex)
    const positionAfter = await invariant.getPosition(account.address, positionIndex)
    assert.equal(
      lowerTickAfter.secondsPerLiquidityOutside,
      upperTickAfter.secondsPerLiquidityOutside
    )
    assert.equal(lowerTickAfter.secondsPerLiquidityOutside, poolAfter.secondsPerLiquidityGlobal)
    assert.equal(positionAfter.secondsPerLiquidityInside, 0n)
  })
})
