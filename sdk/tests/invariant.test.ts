import {
  InvariantError,
  Percentage,
  SqrtPrice
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { getBalance } from '@scio-labs/use-inkathon'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { assertThrowsAsync } from '../src/testUtils'
import { getCodeHash, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { WrappedAZERO } from '../src/wrapped-azero'
import { describe, it } from 'mocha'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

const feeTier = newFeeTier(10000000000n, 1n)

describe('invariant', async function () {
  beforeEach(async function () {
    this.timeout(20000)

    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
  })

  it('should change protocol fee', async () => {
    const newFeeStruct: Percentage = 20000000000n

    await invariant.changeProtocolFee(account, newFeeStruct)
    const newFee = await invariant.getProtocolFee()

    assert.equal(newFee, newFeeStruct)
  })

  it('should add fee tier', async () => {
    const feeTier = newFeeTier(10000000000n, 5n)
    const anotherFeeTier = newFeeTier(20000000000n, 10n)

    await invariant.addFeeTier(account, feeTier)
    let addedFeeTierExists = await invariant.feeTierExist(feeTier)
    const notAddedFeeTierExists = await invariant.feeTierExist(anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers()

    assert.deepEqual(addedFeeTierExists, true)
    assert.deepEqual(notAddedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.addFeeTier(account, anotherFeeTier)
    const addedBeforeFeeTierExists = await invariant.feeTierExist(feeTier)
    addedFeeTierExists = await invariant.feeTierExist(anotherFeeTier)
    feeTiers = await invariant.getFeeTiers()

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
    const notRemovedFeeTierExists = await invariant.feeTierExist(feeTier)
    let removedFeeTierExists = await invariant.feeTierExist(anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers()

    assert.deepEqual(notRemovedFeeTierExists, true)
    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.removeFeeTier(account, feeTier)
    removedFeeTierExists = await invariant.feeTierExist(feeTier)
    const removedBeforeFeeTierExists = await invariant.feeTierExist(anotherFeeTier)
    feeTiers = await invariant.getFeeTiers()

    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(removedBeforeFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 0)
  })

  it('should get tick and check if it is initialized', async () => {
    await invariant.addFeeTier(account, feeTier)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.approve(account, invariant.contract.address.toString(), 1000000000n, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000n, token1Address)

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)
    await invariant.createPosition(account, poolKey, -10n, 10n, 1000000n, pool.sqrtPrice, 0n)

    const lowerTick = await invariant.getTick(poolKey, -10n)
    assert.deepEqual(lowerTick, {
      index: -10n,
      sign: true,
      liquidityChange: 1000000n,
      liquidityGross: 1000000n,
      sqrtPrice: 999500149965006998740209n,
      feeGrowthOutsideX: 0n,
      feeGrowthOutsideY: 0n,
      secondsOutside: lowerTick.secondsOutside
    })
    await assertThrowsAsync(invariant.getTick(poolKey, 0n), InvariantError.TickNotFound)
    const upperTick = await invariant.getTick(poolKey, 10n)
    assert.deepEqual(upperTick, {
      index: 10n,
      sign: false,
      liquidityChange: 1000000n,
      liquidityGross: 1000000n,
      sqrtPrice: 1000500100010000500010000n,
      feeGrowthOutsideX: 0n,
      feeGrowthOutsideY: 0n,
      secondsOutside: upperTick.secondsOutside
    })

    const isLowerTickInitialized = await invariant.isTickInitialized(poolKey, -10n)
    assert.deepEqual(isLowerTickInitialized, true)
    const isInitTickInitialized = await invariant.isTickInitialized(poolKey, 0n)
    assert.deepEqual(isInitTickInitialized, false)
    const isUpperTickInitialized = await invariant.isTickInitialized(poolKey, 10n)
    assert.deepEqual(isUpperTickInitialized, true)
  })

  it('create pool', async () => {
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000000000000000000000000n

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(account, poolKey, initSqrtPrice)
    const pools = await invariant.getPoolKeys(1n, 0n)
    assert.deepEqual(pools[0].length, 1)
    assert.deepEqual(pools[1], 1n)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)
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

  it('create pool x/y and y/x', async () => {
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = 1000000000000000000000000n

    {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await invariant.createPool(account, poolKey, initSqrtPrice)

      const pools = await invariant.getPoolKeys(1n, 0n)
      assert.deepEqual(pools[0].length, 1)
      assert.deepEqual(pools[1], 1n)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
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

      await assertThrowsAsync(invariant.createPool(account, poolKey, initSqrtPrice))
    }
    const pools = await invariant.getPoolKeys(1n, 0n)
    assert.deepEqual(pools[0].length, 1)
    assert.deepEqual(pools[1], 1n)
  })

  it('withdraw all wazero works', async () => {
    const wazero = await WrappedAZERO.deploy(api, Network.Local, account)

    const amount = 10n ** 12n
    await wazero.deposit(account, amount)
    await wazero.approve(account, invariant.contract.address.toString(), amount)

    const AZEROBalanceBefore = await getBalance(api, account.address)
    const parsedAZEROBalanceBefore = BigInt(AZEROBalanceBefore.balance?.toString() ?? 0n)
    const wAZEROBalanceBefore = await wazero.balanceOf(account.address)

    await invariant.withdrawAllWAZERO(account, wazero.contract.address.toString())

    const AZEROBalanceAfter = await getBalance(api, account.address)
    const parsedAZEROBalanceAfter = BigInt(AZEROBalanceAfter.balance?.toString() ?? 0n)
    const wAZEROBalanceAfter = await wazero.balanceOf(account.address)

    assert.isTrue(parsedAZEROBalanceAfter > parsedAZEROBalanceBefore)
    assert.equal(wAZEROBalanceAfter, wAZEROBalanceBefore - amount)
  })

  it('set code works', async () => {
    const codeHash = await getCodeHash(api, invariant.contract.address.toString())

    await invariant.setCode(account, codeHash)

    await invariant.changeProtocolFee(account, 0n)
    const protocolFee = await invariant.getProtocolFee()
    assert.equal(protocolFee, 0n)
  })
})
