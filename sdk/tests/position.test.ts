import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import {
  CreatePositionEvent,
  InvariantError,
  Position,
  SqrtPrice,
  TokenAmount,
  getLiquidityByX,
  isTokenX
} from 'math/math.js'
import { Network } from '../src/network'
import {
  assertThrowsAsync,
  createPositionEventEquals,
  positionEquals,
  removePositionEventEquals
} from '../src/testUtils'
import {
  calculateTokenAmounts,
  deployInvariant,
  deployPSP22,
  initPolkadotApi,
  newFeeTier,
  newPoolKey
} from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

const lowerTickIndex = -20n
const upperTickIndex = 10n
const feeTier = newFeeTier({ v: 6000000000n }, 10n)

let poolKey = newPoolKey(
  token0.contract.address.toString(),
  token1.contract.address.toString(),
  feeTier
)

describe('position', async () => {
  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
    token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

    poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    await token0.approve(account, invariant.contract.address.toString(), 10000000000n)
    await token1.approve(account, invariant.contract.address.toString(), 10000000000n)

    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    const result = await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      { v: 1000000000000n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )

    const expectedCreatePositionEvent: CreatePositionEvent = {
      address: account.address.toString(),
      currentSqrtPrice: { v: 1000000000000000000000000n },
      liquidity: { v: 1000000000000n },
      lowerTick: -20n,
      pool: poolKey,
      upperTick: 10n,
      timestamp: 0n
    }

    createPositionEventEquals(result.events[0], expectedCreatePositionEvent)
  })

  it('create position', async () => {
    const position = await invariant.getPosition(account, account.address, 0n)
    const expectedPosition: Position = {
      poolKey: poolKey,
      liquidity: { v: 1000000000000n },
      lowerTickIndex: lowerTickIndex,
      upperTickIndex: upperTickIndex,
      feeGrowthInsideX: { v: 0n },
      feeGrowthInsideY: { v: 0n },
      lastBlockNumber: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    await positionEquals(position, expectedPosition)
  })
  it('calculate token amounts from position liquidity', async () => {
    const position = await invariant.getPosition(account, account.address, 0n)
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    const providedAmount = 500n
    const { amount: expectedYAmount } = getLiquidityByX(
      500n,
      lowerTickIndex,
      upperTickIndex,
      pool.sqrtPrice,
      false
    )

    const { x, y } = calculateTokenAmounts(pool, position)
    // 1n diffrence in result comes from rounding in `getLiquidityByX`
    assert.deepEqual(x, providedAmount - 1n)
    assert.deepEqual(y, expectedYAmount)
  })
  it('remove position', async () => {
    {
      const result = await invariant.removePosition(account, 0n)

      const expectedRemovePositionEvent = {
        address: account.address.toString(),
        currentSqrtPrice: { v: 1000000000000000000000000n },
        liquidity: { v: 1000000000000n },
        lowerTick: -20n,
        pool: poolKey,
        upperTick: 10n,
        timestamp: 0n
      }

      removePositionEventEquals(result.events[0], expectedRemovePositionEvent)

      assertThrowsAsync(
        invariant.getPosition(account, account.address, 0n),
        InvariantError.PositionNotFound
      )
      const positions = await invariant.getPositions(account, account.address)
      assert.deepEqual(positions.length, 0)
    }
    {
      assertThrowsAsync(
        invariant.getTick(account, poolKey, lowerTickIndex),
        InvariantError.TickNotFound
      )

      assertThrowsAsync(
        invariant.getTick(account, poolKey, upperTickIndex),
        InvariantError.TickNotFound
      )

      const isLowerTickInitialized = await invariant.isTickInitialized(
        account,
        poolKey,
        lowerTickIndex
      )
      assert.exists(!isLowerTickInitialized)

      const isUpperTickInitialized = await invariant.isTickInitialized(
        account,
        poolKey,
        upperTickIndex
      )

      assert.exists(!isUpperTickInitialized)
    }
  })

  it('transfer position', async () => {
    {
      const positionOwner = keyring.addFromUri('//Alice')
      const receiver = keyring.addFromUri('//Bob')
      await invariant.transferPosition(positionOwner, 0n, receiver.address)

      assertThrowsAsync(
        invariant.getPosition(positionOwner, positionOwner.address, 0n),
        InvariantError.PositionNotFound
      )
      const position = await invariant.getPosition(receiver, receiver.address, 0n)
      const expectedPosition: Position = {
        poolKey: poolKey,
        liquidity: { v: 1000000000000n },
        lowerTickIndex: lowerTickIndex,
        upperTickIndex: upperTickIndex,
        feeGrowthInsideX: { v: 0n },
        feeGrowthInsideY: { v: 0n },
        lastBlockNumber: 0n,
        tokensOwedX: 0n,
        tokensOwedY: 0n
      }
      await positionEquals(position, expectedPosition)
    }
  })

  it('claim fee', async () => {
    const [tokenX, tokenY] = isTokenX(
      token0.contract.address.toString(),
      token1.contract.address.toString()
    )
      ? [token0, token1]
      : [token1, token0]

    {
      const amount: TokenAmount = 1000n
      const swapper = keyring.addFromUri('//Bob')

      await tokenX.mint(swapper, amount)
      await tokenX.approve(swapper, invariant.contract.address.toString(), amount)

      const poolBefore = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )

      const targetSqrtPrice: SqrtPrice = { v: 15258932000000000000n }
      await invariant.swap(swapper, poolKey, true, amount, true, targetSqrtPrice)

      const poolAfter = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      const swapperX = await tokenX.balanceOf(swapper, swapper.address)
      const swapperY = await tokenY.balanceOf(swapper, swapper.address)

      assert.equal(swapperX, 0n)
      assert.equal(swapperY, 993n)

      const invariantX = await tokenX.balanceOf(account, invariant.contract.address.toString())
      const invariantY = await tokenY.balanceOf(account, invariant.contract.address.toString())

      assert.equal(invariantX, 1500n)
      assert.equal(invariantY, 7n)

      assert.deepEqual(poolAfter.liquidity, poolBefore.liquidity)
      assert.notDeepEqual(poolAfter.sqrtPrice, poolBefore.sqrtPrice)
      assert.deepEqual(poolAfter.currentTickIndex, lowerTickIndex)
      assert.deepEqual(poolAfter.feeGrowthGlobalX, { v: 50000000000000000000000n })
      assert.deepEqual(poolAfter.feeGrowthGlobalY, { v: 0n })
      assert.deepEqual(poolAfter.feeProtocolTokenX, 1n)
      assert.deepEqual(poolAfter.feeProtocolTokenY, 0n)
    }
    {
      const positionOwnerBeforeX = await tokenX.balanceOf(account, account.address)
      const invariantBeforeX = await tokenX.balanceOf(
        account,
        invariant.contract.address.toString()
      )

      await invariant.claimFee(account, 0n)

      const positionOwnerAfterX = await tokenX.balanceOf(account, account.address)

      const invariantAfterX = await tokenX.balanceOf(account, invariant.contract.address.toString())

      const position = await invariant.getPosition(account, account.address, 0n)
      const pool = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      const expectedTokensClaimed = 5n

      assert.deepEqual(positionOwnerAfterX - expectedTokensClaimed, positionOwnerBeforeX)
      assert.deepEqual(invariantAfterX + expectedTokensClaimed, invariantBeforeX)

      assert.deepEqual(position.feeGrowthInsideX, pool.feeGrowthGlobalX)
      assert.deepEqual(position.tokensOwedX, 0n)
    }
  })
})
