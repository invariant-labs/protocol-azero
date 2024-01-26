import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import {
  CreatePositionEvent,
  CrossTickEvent,
  RemovePositionEvent,
  SwapEvent,
  getGlobalMinSqrtPrice,
  toPercentage,
  toSqrtPrice
} from 'math/math'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { InvariantEvent } from '../src/schema'
import {
  createPositionEventEquals,
  crossTickEventEquals,
  removePositionEventEquals,
  swapEventEquals
} from '../src/testUtils'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const feeTier = newFeeTier(10000000000n, 1n)

let invariant = await Invariant.deploy(api, Network.Local, account, toPercentage(1n, 2n))
let token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

const psp22 = await PSP22.load(api, Network.Local, token0Address)

describe('events', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, toPercentage(1n, 2n))
    token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)
    await invariant.createPool(account, poolKey, toSqrtPrice(1n, 0n))

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)
  })

  it('create position event', async () => {
    let wasFired = false

    const expectedCreatePositionEvent: CreatePositionEvent = {
      address: account.address.toString(),
      currentSqrtPrice: toSqrtPrice(1n, 0n),
      liquidity: 1000000000000n,
      lowerTick: -10n,
      pool: poolKey,
      upperTick: 10n,
      timestamp: 0n
    }

    invariant.on(InvariantEvent.CreatePositionEvent, (event: CreatePositionEvent) => {
      wasFired = true

      createPositionEventEquals(event, expectedCreatePositionEvent)
    })

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)

    const result = await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(result.events.length, 1)
    createPositionEventEquals(result.events[0], expectedCreatePositionEvent)
    assert.deepEqual(wasFired, true)
  })

  it('cross tick and swap event', async () => {
    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)

    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    await invariant.createPosition(
      account,
      poolKey,
      -30n,
      -10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    await invariant.createPosition(
      account,
      poolKey,
      -50n,
      -30n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    let wasSwapEventFired = false
    let wasCrossTickEventFired = false

    const expectedCrossTickEvent: CrossTickEvent = {
      address: account.address,
      pool: poolKey,
      timestamp: 0n,
      indexes: [-10n, -30n]
    }

    const expectedSwapEvent: SwapEvent = {
      address: account.address,
      pool: poolKey,
      amountIn: 2500n,
      amountOut: 2464n,
      fee: 27n,
      startSqrtPrice: 1000000000000000000000000n,
      targetSqrtPrice: 997534045508480530459903n,
      xToY: true,
      timestamp: 0n
    }

    invariant.on(InvariantEvent.CrossTickEvent, (event: CrossTickEvent) => {
      wasCrossTickEventFired = true

      crossTickEventEquals(event, expectedCrossTickEvent)
    })

    invariant.on(InvariantEvent.SwapEvent, (event: SwapEvent) => {
      wasSwapEventFired = true

      swapEventEquals(event, expectedSwapEvent)
    })

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)

    const result = await invariant.swap(
      account,
      poolKey,
      true,
      2500n,
      true,
      getGlobalMinSqrtPrice()
    )

    assert.deepEqual(result.events.length, 2)
    crossTickEventEquals(result.events[0] as CrossTickEvent, expectedCrossTickEvent)
    swapEventEquals(result.events[1] as SwapEvent, expectedSwapEvent)
    assert.deepEqual(wasCrossTickEventFired, true)
    assert.deepEqual(wasSwapEventFired, true)
  })

  it('remove position event', async () => {
    let wasFired = false

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)

    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    const expectedRemovePositionEvent: RemovePositionEvent = {
      address: account.address.toString(),
      currentSqrtPrice: toSqrtPrice(1n, 0n),
      liquidity: 1000000000000n,
      lowerTick: -10n,
      pool: poolKey,
      upperTick: 10n,
      timestamp: 0n
    }

    invariant.on(InvariantEvent.RemovePositionEvent, (event: RemovePositionEvent) => {
      wasFired = true

      createPositionEventEquals(event, expectedRemovePositionEvent)
    })

    const result = await invariant.removePosition(account, 0n)

    assert.deepEqual(result.events.length, 1)
    removePositionEventEquals(result.events[0], expectedRemovePositionEvent)
    assert.deepEqual(wasFired, true)
  })

  it('on and off methods', async () => {
    let timesFired = 0

    const handler = () => {
      timesFired++
    }

    invariant.on(InvariantEvent.CreatePositionEvent, handler)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000000n)

    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(timesFired, 1)

    invariant.off(InvariantEvent.CreatePositionEvent, handler)

    await invariant.createPosition(
      account,
      poolKey,
      -50n,
      50n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(timesFired, 1)

    invariant.on(InvariantEvent.CreatePositionEvent, handler)

    await invariant.createPosition(
      account,
      poolKey,
      -40n,
      40n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(timesFired, 2)
  })
})
