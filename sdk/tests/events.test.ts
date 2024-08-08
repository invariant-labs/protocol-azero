import {
  CreatePositionEvent,
  CrossTickEvent,
  RemovePositionEvent,
  SwapEvent,
  getGlobalMinSqrtPrice,
  toPercentage,
  toSqrtPrice
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import 'mocha'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { InvariantEvent } from '../src/schema'
import { objectEquals } from '../src/testUtils'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { describe, it } from 'mocha'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const feeTier = newFeeTier(10000000000n, 1n)

let invariant = await Invariant.deploy(api, Network.Local, account, toPercentage(1n, 2n))
let token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

const psp22 = await PSP22.load(api, Network.Local)

describe('events', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, toPercentage(1n, 2n))
    token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)
    await invariant.createPool(account, poolKey, toSqrtPrice(1n, 0n))

    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token1Address
    )
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

      objectEquals(event, expectedCreatePositionEvent, ['timestamp'])
    })

    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token1Address
    )

    const result = await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      1000000000000n,
      toSqrtPrice(1n, 0n),
      0n
    )

    assert.deepEqual(result.events.length, 5)
    objectEquals(result.events[4], expectedCreatePositionEvent, ['timestamp'])
    assert.deepEqual(wasFired, true)
  })

  it('cross tick and swap event', async () => {
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token1Address
    )

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
      targetSqrtPrice: 997534045508785821944214n,
      xToY: true,
      timestamp: 0n
    }

    invariant.on(InvariantEvent.CrossTickEvent, (event: CrossTickEvent) => {
      wasCrossTickEventFired = true

      objectEquals(event, expectedCrossTickEvent, ['timestamp'])
    })

    invariant.on(InvariantEvent.SwapEvent, (event: SwapEvent) => {
      wasSwapEventFired = true

      objectEquals(event, expectedSwapEvent, ['timestamp'])
    })

    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token1Address
    )

    const result = await invariant.swap(
      account,
      poolKey,
      true,
      2500n,
      true,
      getGlobalMinSqrtPrice()
    )

    assert.deepEqual(result.events.length, 5)
    objectEquals(result.events[0] as CrossTickEvent, expectedCrossTickEvent, ['timestamp'])
    objectEquals(result.events[4] as SwapEvent, expectedSwapEvent, ['timestamp'])
    assert.deepEqual(wasCrossTickEventFired, true)
    assert.deepEqual(wasSwapEventFired, true)
  })

  it('remove position event', async () => {
    let wasFired = false

    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token1Address
    )

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

      objectEquals(event, expectedRemovePositionEvent, ['timestamp'])
    })

    const result = await invariant.removePosition(account, 0n)

    assert.deepEqual(result.events.length, 3)
    objectEquals(result.events[2], expectedRemovePositionEvent, ['timestamp'])
    assert.deepEqual(wasFired, true)
  })

  it('on and off methods', async () => {
    let timesFired = 0

    const handler = () => {
      timesFired++
    }

    invariant.on(InvariantEvent.CreatePositionEvent, handler)

    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1000000000000n,
      token1Address
    )

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
