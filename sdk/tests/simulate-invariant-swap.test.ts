import {
  SwapEvent,
  getMaxSqrtPrice,
  getMinSqrtPrice
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { expect } from 'chai'
import { MAX_SQRT_PRICE, MIN_SQRT_PRICE } from '../src/consts'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { assertThrowsAsync } from '../src/testUtils'
import {
  delay,
  filterTickmap,
  filterTicks,
  initPolkadotApi,
  newFeeTier,
  newPoolKey,
  simulateInvariantSwap
} from '../src/utils'
import { describe, it } from 'mocha'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

const protocolFee = 10000000000n

let invariant = await Invariant.deploy(api, Network.Local, account, protocolFee)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

const feeTier = newFeeTier(10000000000n, 1n)

describe('simulateInvariantSwap', async () => {
  beforeEach(async function () {
    this.timeout(20000)
    await api.disconnect()
    await delay(2000)
    await api.connect()

    invariant = await Invariant.deploy(api, Network.Local, account, protocolFee)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    await invariant.addFeeTier(account, feeTier)

    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      10000000000000n,
      token0Address
    )
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      10000000000000n,
      token1Address
    )

    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      10000000000000n,
      1000000000000000000000000n,
      0n
    )

    await psp22.approve(account, invariant.contract.address.toString(), 1000000000n, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 1000000000n, token1Address)
  })
  context('reaches price limit', async () => {
    it('X to Y by amount in', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)

      const amountIn = 6000n
      const byAmountIn = true
      const xToY = true

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })

    it('Y to X by amount in', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 6000n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })

    it('Y to X', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 5000n
      const byAmountIn = false
      const xToY = false
      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })

    it('X to Y', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)
      const amountIn = 5000n
      const byAmountIn = false
      const xToY = true

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.globalInsufficientLiquidity).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)

      await assertThrowsAsync(
        invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, sqrtPriceLimit)
      )
    })
  })

  context('matches the price', async () => {
    it('X to Y by amount in', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4999n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swap = await invariant.swap(
        account,
        poolKey,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )
      // TODO: fix events not being filtered properly (possibly a polkadot issue)
      expect(swap.events.length).to.equal(4)
      const swapResult = swap.events[3] as SwapEvent

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(0)
    })

    it('Y to X by amount in', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4999n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swap = await invariant.swap(
        account,
        poolKey,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      // TODO: fix events not being filtered properly (possibly a polkadot issue)
      expect(swap.events.length).to.equal(4)
      const swapResult = swap.events[3] as SwapEvent

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)

      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(0)
    })

    it('Y to X', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4888n
      const byAmountIn = false
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swap = await invariant.swap(
        account,
        poolKey,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      // TODO: fix events not being filtered properly (possibly a polkadot issue)
      expect(swap.events.length).to.equal(4)
      const swapResult = swap.events[3] as SwapEvent

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)

      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(0)
    })

    it('X to Y', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)

      const amountIn = 4888n
      const byAmountIn = false
      const xToY = true

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      const swap = await invariant.swap(
        account,
        poolKey,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      // TODO: fix events not being filtered properly (possibly a polkadot issue)
      expect(swap.events.length).to.equal(4)
      const swapResult = swap.events[3] as SwapEvent

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.stateOutdated).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(swapResult.amountIn).to.equal(simulation.amountIn)
      expect(swapResult.amountOut).to.equal(simulation.amountOut)
      expect(swapResult.startSqrtPrice).to.equal(simulation.startSqrtPrice)
      expect(swapResult.targetSqrtPrice).to.equal(simulation.targetSqrtPrice)
      expect(swapResult.fee).to.equal(simulation.fee)
      expect(simulation.crossedTicks.length).to.equal(0)
    })
  })

  context('outdated data in', async () => {
    it('pool', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 6000n
      const byAmountIn = true
      const xToY = false

      await invariant.createPosition(
        account,
        poolKey,
        -10n,
        10n,
        10000000000000n,
        1000000000000000000000000n,
        0n
      )

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.stateOutdated).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(0)
    })

    it('tickmap', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)
      const pool = await invariant.getPool(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMaxSqrtPrice(feeTier.tickSpacing)
      const amountIn = 6000n
      const byAmountIn = true
      const xToY = false

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      await invariant.createPosition(
        account,
        poolKey,
        -20n,
        10n,
        10000000000000n,
        1000000000000000000000000n,
        0n
      )

      const ticks = filterTicks(
        await invariant.getAllLiquidityTicks(poolKey, tickmap),
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.stateOutdated).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(0)
    })

    it('ticks', async () => {
      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)
      const amountIn = 20000n
      const byAmountIn = true
      const xToY = true

      await invariant.createPosition(
        account,
        poolKey,
        -20n,
        10n,
        10000000000000n,
        1000000000000000000000000n,
        0n
      )

      const pool = await invariant.getPool(token0Address, token1Address, feeTier)
      const ticks = filterTicks(
        await invariant.getLiquidityTicks(poolKey, [10n, -10n]),
        pool.currentTickIndex,
        xToY
      )

      const tickmap = filterTickmap(
        await invariant.getFullTickmap(poolKey),
        poolKey.feeTier.tickSpacing,
        pool.currentTickIndex,
        xToY
      )

      const simulation = simulateInvariantSwap(
        tickmap,
        feeTier,
        pool,
        ticks,
        xToY,
        amountIn,
        byAmountIn,
        sqrtPriceLimit
      )

      expect(simulation.globalInsufficientLiquidity).to.equal(false)
      expect(simulation.maxTicksCrossed).to.equal(false)
      expect(simulation.stateOutdated).to.equal(true)
      expect(simulation.crossedTicks.length).to.equal(1)
    })
  })
  it('max ticks crossed', async function () {
    this.timeout(2000000)
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    const sqrtPriceLimit = getMinSqrtPrice(feeTier.tickSpacing)
    const amountIn = 1000000n
    const byAmountIn = true
    const xToY = true

    const mintAmount = 1n << 120n
    await psp22.mint(account, mintAmount, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), mintAmount, token0Address)
    await psp22.mint(account, mintAmount, token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), mintAmount, token1Address)

    const liquidityDelta = 10000000n * 10n ** 6n
    const spotSqrtPrice = 1000000000000000000000000n
    const slippageTolerance = 0n

    const indexes: bigint[] = []

    for (let i = -256n; i < 5; i += 1n) {
      indexes.push(i + 1n)
      await invariant.createPosition(
        account,
        poolKey,
        i,
        i + 1n,
        liquidityDelta,
        spotSqrtPrice,
        slippageTolerance
      )
    }

    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const tickmap = filterTickmap(
      await invariant.getFullTickmap(poolKey),
      poolKey.feeTier.tickSpacing,
      pool.currentTickIndex,
      xToY
    )

    const ticks = filterTicks(
      await invariant.getAllLiquidityTicks(poolKey, tickmap),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      sqrtPriceLimit
    )
    expect(simulation.crossedTicks.length).to.equal(129)
    expect(simulation.globalInsufficientLiquidity).to.equal(false)
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(true)
  })

  it('max token amount - X to Y - amount in', async () => {
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = 2n ** 128n - 1n
    const byAmountIn = true
    const xToY = true

    const tickmap = filterTickmap(
      await invariant.getFullTickmap(poolKey),
      poolKey.feeTier.tickSpacing,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getAllLiquidityTicks(poolKey, tickmap),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MIN_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, MIN_SQRT_PRICE)
    )
  })

  it('max token amount - X to Y - amount out', async () => {
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = 2n ** 128n - 1n
    const byAmountIn = false
    const xToY = true

    const tickmap = filterTickmap(
      await invariant.getFullTickmap(poolKey),
      poolKey.feeTier.tickSpacing,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getAllLiquidityTicks(poolKey, tickmap),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MIN_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, MIN_SQRT_PRICE)
    )
  })

  it('max token amount - Y to X - amount in', async () => {
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = 2n ** 128n - 1n
    const byAmountIn = true
    const xToY = false

    const tickmap = filterTickmap(
      await invariant.getFullTickmap(poolKey),
      poolKey.feeTier.tickSpacing,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getAllLiquidityTicks(poolKey, tickmap),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MAX_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, MAX_SQRT_PRICE)
    )
  })

  it('max token amount - Y to X - amount out', async () => {
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)
    const pool = await invariant.getPool(token0Address, token1Address, feeTier)

    const amountIn = 2n ** 128n - 1n
    const byAmountIn = false
    const xToY = false

    const tickmap = filterTickmap(
      await invariant.getFullTickmap(poolKey),
      poolKey.feeTier.tickSpacing,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getAllLiquidityTicks(poolKey, tickmap),
      pool.currentTickIndex,
      xToY
    )

    const simulation = simulateInvariantSwap(
      tickmap,
      feeTier,
      pool,
      ticks,
      xToY,
      amountIn,
      byAmountIn,
      MAX_SQRT_PRICE
    )
    expect(simulation.stateOutdated).to.equal(false)
    expect(simulation.maxTicksCrossed).to.equal(false)
    expect(simulation.globalInsufficientLiquidity).to.equal(true)
    expect(simulation.crossedTicks.length).to.equal(1)

    await assertThrowsAsync(
      invariant.swap(account, poolKey, xToY, amountIn, byAmountIn, MAX_SQRT_PRICE)
    )
  })
})
