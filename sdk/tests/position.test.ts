import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { assertThrowsAsync, objectEquals } from '../src/testUtils'
import { calculateTokenAmounts, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import {
  CreatePositionEvent,
  InvariantError,
  Position,
  SqrtPrice,
  TokenAmount,
  getLiquidityByX,
  isTokenX
} from '../src/wasm/pkg/invariant_a0_wasm'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

const lowerTickIndex = -20n
const upperTickIndex = 10n
const feeTier = newFeeTier(6000000000n, 10n)

let poolKey = newPoolKey(token0Address, token1Address, feeTier)

describe('position', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)

    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

    const result = await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      1000000000000n,
      pool.sqrtPrice,
      0n
    )

    const expectedCreatePositionEvent: CreatePositionEvent = {
      address: account.address.toString(),
      currentSqrtPrice: 1000000000000000000000000n,
      liquidity: 1000000000000n,
      lowerTick: -20n,
      pool: poolKey,
      upperTick: 10n,
      timestamp: 0n
    }

    objectEquals(result.events[0], expectedCreatePositionEvent, ['timestamp'])
  })

  it('create position', async () => {
    const position = await invariant.getPosition(account, account.address, 0n)
    const expectedPosition: Position = {
      poolKey: poolKey,
      liquidity: 1000000000000n,
      lowerTickIndex: lowerTickIndex,
      upperTickIndex: upperTickIndex,
      feeGrowthInsideX: 0n,
      feeGrowthInsideY: 0n,
      lastBlockNumber: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    await objectEquals(position, expectedPosition, ['lastBlockNumber'])
  })
  it('calculate token amounts from position liquidity', async () => {
    const position = await invariant.getPosition(account, account.address, 0n)
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

    const providedAmount = 500n
    const { amount: expectedYAmount } = getLiquidityByX(
      500n,
      lowerTickIndex,
      upperTickIndex,
      pool.sqrtPrice,
      false
    )

    const [x, y] = calculateTokenAmounts(pool, position)
    // 1n diffrence in result comes from rounding in `getLiquidityByX`
    assert.deepEqual(x, providedAmount - 1n)
    assert.deepEqual(y, expectedYAmount)
  })
  it('remove position', async () => {
    {
      const result = await invariant.removePosition(account, 0n)

      const expectedRemovePositionEvent = {
        address: account.address.toString(),
        currentSqrtPrice: 1000000000000000000000000n,
        liquidity: 1000000000000n,
        lowerTick: -20n,
        pool: poolKey,
        upperTick: 10n,
        timestamp: 0n
      }

      objectEquals(result.events[0], expectedRemovePositionEvent, ['timestamp'])

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
        liquidity: 1000000000000n,
        lowerTickIndex: lowerTickIndex,
        upperTickIndex: upperTickIndex,
        feeGrowthInsideX: 0n,
        feeGrowthInsideY: 0n,
        lastBlockNumber: 0n,
        tokensOwedX: 0n,
        tokensOwedY: 0n
      }
      await objectEquals(position, expectedPosition, ['lastBlockNumber'])
    }
  })

  it('claim fee', async () => {
    const [tokenX, tokenY] = isTokenX(token0Address, token1Address)
      ? [token0Address, token1Address]
      : [token1Address, token0Address]

    {
      const amount: TokenAmount = 1000n
      const swapper = keyring.addFromUri('//Bob')

      await psp22.setContractAddress(tokenX)
      await psp22.mint(swapper, amount)
      await psp22.approve(swapper, invariant.contract.address.toString(), amount)

      const poolBefore = await invariant.getPool(account, token0Address, token1Address, feeTier)

      const targetSqrtPrice: SqrtPrice = 15258932000000000000n
      await invariant.swap(swapper, poolKey, true, amount, true, targetSqrtPrice)

      const poolAfter = await invariant.getPool(account, token0Address, token1Address, feeTier)
      await psp22.setContractAddress(tokenX)
      const swapperX = await psp22.balanceOf(swapper, swapper.address)
      await psp22.setContractAddress(tokenY)
      const swapperY = await psp22.balanceOf(swapper, swapper.address)

      assert.equal(swapperX, 0n)
      assert.equal(swapperY, 993n)

      await psp22.setContractAddress(tokenX)
      const invariantX = await psp22.balanceOf(account, invariant.contract.address.toString())
      await psp22.setContractAddress(tokenY)
      const invariantY = await psp22.balanceOf(account, invariant.contract.address.toString())

      assert.equal(invariantX, 1500n)
      assert.equal(invariantY, 7n)

      assert.deepEqual(poolAfter.liquidity, poolBefore.liquidity)
      assert.notDeepEqual(poolAfter.sqrtPrice, poolBefore.sqrtPrice)
      assert.deepEqual(poolAfter.currentTickIndex, lowerTickIndex)
      assert.deepEqual(poolAfter.feeGrowthGlobalX, 50000000000000000000000n)
      assert.deepEqual(poolAfter.feeGrowthGlobalY, 0n)
      assert.deepEqual(poolAfter.feeProtocolTokenX, 1n)
      assert.deepEqual(poolAfter.feeProtocolTokenY, 0n)
    }
    {
      await psp22.setContractAddress(tokenX)
      const positionOwnerBeforeX = await psp22.balanceOf(account, account.address)
      const invariantBeforeX = await psp22.balanceOf(account, invariant.contract.address.toString())

      await invariant.claimFee(account, 0n)

      await psp22.setContractAddress(tokenX)
      const positionOwnerAfterX = await psp22.balanceOf(account, account.address)

      const invariantAfterX = await psp22.balanceOf(account, invariant.contract.address.toString())

      const position = await invariant.getPosition(account, account.address, 0n)
      const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
      const expectedTokensClaimed = 5n

      assert.deepEqual(positionOwnerAfterX - expectedTokensClaimed, positionOwnerBeforeX)
      assert.deepEqual(invariantAfterX + expectedTokensClaimed, invariantBeforeX)

      assert.deepEqual(position.feeGrowthInsideX, pool.feeGrowthGlobalX)
      assert.deepEqual(position.tokensOwedX, 0n)
    }
  })
})
