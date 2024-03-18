import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { InvariantTx } from '../src/schema'
import { assertThrowsAsync } from '../src/testUtils'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { InvariantError, Percentage, SqrtPrice } from '../src/wasm/pkg/invariant_a0_wasm'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

const feeTier = newFeeTier(10000000000n, 1n)

describe('invariant', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
  })

  it('should change protocol fee', async () => {
    const newFeeStruct: Percentage = 20000000000n

    await invariant.changeProtocolFee(account, newFeeStruct)
    const newFee = await invariant.getProtocolFee(account)

    assert.equal(newFee, newFeeStruct)
  })

  it('should add fee tier', async () => {
    const feeTier = newFeeTier(10000000000n, 5n)
    const anotherFeeTier = newFeeTier(20000000000n, 10n)

    await invariant.addFeeTier(account, feeTier)
    let addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    const notAddedFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(addedFeeTierExists, true)
    assert.deepEqual(notAddedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.addFeeTier(account, anotherFeeTier)
    const addedBeforeFeeTierExists = await invariant.feeTierExist(account, feeTier)
    addedFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(addedBeforeFeeTierExists, true)
    assert.deepEqual(addedFeeTierExists, true)
    assert.deepEqual(feeTiers.length, 2)
  })

  it('should remove fee tier', async () => {
    const feeTier = newFeeTier(10000000000n, 5n)
    const anotherFeeTier = newFeeTier(20000000000n, 10n)

    await invariant.addFeeTier(account, feeTier)
    await invariant.addFeeTier(account, anotherFeeTier)

    await invariant.removeFeeTier(account, anotherFeeTier)
    const notRemovedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    let removedFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(notRemovedFeeTierExists, true)
    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.removeFeeTier(account, feeTier)
    removedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    const removedBeforeFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(removedBeforeFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 0)
  })

  it('should get tick and check if it is initialized', async () => {
    await invariant.addFeeTier(account, feeTier)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000n)

    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
    await invariant.createPosition(account, poolKey, -10n, 10n, 1000000n, pool.sqrtPrice, 0n)

    const lowerTick = await invariant.getTick(account, poolKey, -10n)
    assert.deepEqual(lowerTick, {
      index: -10n,
      sign: true,
      liquidityChange: 1000000n,
      liquidityGross: 1000000n,
      sqrtPrice: 999500149965000000000000n,
      feeGrowthOutsideX: 0n,
      feeGrowthOutsideY: 0n,
      secondsOutside: lowerTick.secondsOutside
    })
    await assertThrowsAsync(invariant.getTick(account, poolKey, 0n), InvariantError.TickNotFound)
    const upperTick = await invariant.getTick(account, poolKey, 10n)
    assert.deepEqual(upperTick, {
      index: 10n,
      sign: false,
      liquidityChange: 1000000n,
      liquidityGross: 1000000n,
      sqrtPrice: 1000500100010000000000000n,
      feeGrowthOutsideX: 0n,
      feeGrowthOutsideY: 0n,
      secondsOutside: upperTick.secondsOutside
    })

    const isLowerTickInitialized = await invariant.isTickInitialized(account, poolKey, -10n)
    assert.deepEqual(isLowerTickInitialized, true)
    const isInitTickInitialized = await invariant.isTickInitialized(account, poolKey, 0n)
    assert.deepEqual(isInitTickInitialized, false)
    const isUpperTickInitialized = await invariant.isTickInitialized(account, poolKey, 10n)
    assert.deepEqual(isUpperTickInitialized, true)
  })

  it('create pool', async () => {
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000000000000000000000000n

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(account, poolKey, initSqrtPrice)
    const pools = await invariant.getPools(account, 1n, 0n)
    assert.deepEqual(pools.length, 1)
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
    assert.deepEqual(pool, {
      liquidity: 0n,
      sqrtPrice: 1000000000000000000000000n,
      currentTickIndex: 0n,
      feeGrowthGlobalX: 0n,
      feeGrowthGlobalY: 0n,
      feeProtocolTokenX: 0n,
      feeProtocolTokenY: 0n,
      startTimestamp: pool.startTimestamp,
      lastTimestamp: pool.lastTimestamp,
      feeReceiver: pool.feeReceiver
    })
  })

  it('attempt to try create pool with wrong tick sqrtPrice relationship', async () => {
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000175003749000000000000n

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    assertThrowsAsync(
      invariant.createPool(account, poolKey, initSqrtPrice),
      InvariantError.InvalidInitTick
    )
  })

  it('create pool x/y and y/x', async () => {
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000000000000000000000000n

    {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await invariant.createPool(account, poolKey, initSqrtPrice)

      const pools = await invariant.getPools(account, 1n, 0n)
      assert.deepEqual(pools.length, 1)
      const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
      assert.deepEqual(pool, {
        liquidity: 0n,
        sqrtPrice: 1000000000000000000000000n,
        currentTickIndex: 0n,
        feeGrowthGlobalX: 0n,
        feeGrowthGlobalY: 0n,
        feeProtocolTokenX: 0n,
        feeProtocolTokenY: 0n,
        startTimestamp: pool.startTimestamp,
        lastTimestamp: pool.lastTimestamp,
        feeReceiver: pool.feeReceiver
      })
    }
    {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await assertThrowsAsync(
        invariant.createPool(account, poolKey, initSqrtPrice),
        InvariantTx.CreatePool
      )
    }
    const pools = await invariant.getPools(account, 1n, 0n)
    assert.deepEqual(pools.length, 1)
  })
})
