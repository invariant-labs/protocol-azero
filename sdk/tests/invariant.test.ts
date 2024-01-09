import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import {
  InvariantError,
  Position,
  SqrtPrice,
  TokenAmount,
  newFeeTier,
  newPoolKey
} from 'math/math.js'
import { Network } from '../src/network'
import { InvariantTx } from '../src/schema'
import { assertThrowsAsync, positionEquals } from '../src/testUtils'
import { _newPoolKey, deployInvariant, deployPSP22, initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const testAccount = await keyring.addFromUri('//Bob')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

describe('invariant', async () => {
  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
    token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
  })

  it('should change protocol fee', async () => {
    const newFeeStruct = { v: 20000000000n }

    await invariant.changeProtocolFee(account, newFeeStruct)
    const newFee = await invariant.getProtocolFee(account)

    assert.equal(newFee.v, newFeeStruct.v)
  })

  it('should add fee tier', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 5)
    const anotherFeeTier = newFeeTier({ v: 20000000000n }, 10)

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
    const feeTier = newFeeTier({ v: 10000000000n }, 5)
    const anotherFeeTier = newFeeTier({ v: 20000000000n }, 10)

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
    const feeTier = newFeeTier({ v: 10000000000n }, 1)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    await token0.approve(account, invariant.contract.address.toString(), 1000000000n)
    await token1.approve(account, invariant.contract.address.toString(), 1000000000n)

    const poolKey = _newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      { v: 1000000n },
      pool.sqrtPrice,
      pool.sqrtPrice
    )

    const lowerTick = await invariant.getTick(account, poolKey, -10n)
    assert.deepEqual(lowerTick, {
      index: -10n,
      sign: true,
      liquidityChange: { v: 1000000n },
      liquidityGross: { v: 1000000n },
      sqrtPrice: { v: 999500149965000000000000n },
      feeGrowthOutsideX: { v: 0n },
      feeGrowthOutsideY: { v: 0n },
      secondsOutside: lowerTick.secondsOutside
    })
    await assertThrowsAsync(invariant.getTick(account, poolKey, 0n), InvariantError.TickNotFound)
    const upperTick = await invariant.getTick(account, poolKey, 10n)
    assert.deepEqual(upperTick, {
      index: 10n,
      sign: false,
      liquidityChange: { v: 1000000n },
      liquidityGross: { v: 1000000n },
      sqrtPrice: { v: 1000500100010000000000000n },
      feeGrowthOutsideX: { v: 0n },
      feeGrowthOutsideY: { v: 0n },
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
    const feeTier = newFeeTier({ v: 10000000000n }, 1)
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = { v: 1000000000000000000000000n }
    const initTick = 0n

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      initSqrtPrice,
      initTick
    )
    const pools = await invariant.getPools(account)
    assert.deepEqual(pools.length, 1)
    const pool = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    assert.deepEqual(pool, {
      liquidity: { v: 0n },
      sqrtPrice: { v: 1000000000000000000000000n },
      currentTickIndex: 0n,
      feeGrowthGlobalX: { v: 0n },
      feeGrowthGlobalY: { v: 0n },
      feeProtocolTokenX: 0n,
      feeProtocolTokenY: 0n,
      startTimestamp: pool.startTimestamp,
      lastTimestamp: pool.lastTimestamp,
      feeReceiver: pool.feeReceiver
    })
  })
  it('create pool x/y and y/x', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 1)
    await invariant.addFeeTier(account, feeTier)
    const addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const initSqrtPrice: SqrtPrice = { v: 1000000000000000000000000n }
    const initTick = 0n

    {
      await invariant.createPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier,
        initSqrtPrice,
        initTick
      )

      const pools = await invariant.getPools(account)
      assert.deepEqual(pools.length, 1)
      const pool = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      assert.deepEqual(pool, {
        liquidity: { v: 0n },
        sqrtPrice: { v: 1000000000000000000000000n },
        currentTickIndex: 0n,
        feeGrowthGlobalX: { v: 0n },
        feeGrowthGlobalY: { v: 0n },
        feeProtocolTokenX: 0n,
        feeProtocolTokenY: 0n,
        startTimestamp: pool.startTimestamp,
        lastTimestamp: pool.lastTimestamp,
        feeReceiver: pool.feeReceiver
      })
    }
    {
      await assertThrowsAsync(
        invariant.createPool(
          account,
          token1.contract.address.toString(),
          token0.contract.address.toString(),
          feeTier,
          initSqrtPrice,
          initTick
        ),
        InvariantTx.CreatePool
      )
    }
    const pools = await invariant.getPools(account)
    assert.deepEqual(pools.length, 1)
  })

  describe('positions', async () => {
    beforeEach(async () => {
      const feeTier = newFeeTier({ v: 10000000000n }, 1)

      await invariant.addFeeTier(account, feeTier)

      await invariant.createPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier,
        { v: 1000000000000000000000000n },
        0n
      )

      await token0.approve(account, invariant.contract.address.toString(), 10000000000000n)
      await token1.approve(account, invariant.contract.address.toString(), 10000000000000n)

      const poolKey = newPoolKey(
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )

      await invariant.createPosition(
        account,
        poolKey,
        -10n,
        10n,
        { v: 10000000000000n },
        { v: 1000000000000000000000000n },
        { v: 1000000000000000000000000n }
      )

      await token0.approve(account, invariant.contract.address.toString(), 1000000000n)
      await token1.approve(account, invariant.contract.address.toString(), 1000000000n)

      await invariant.swap(account, poolKey, true, 4999n, true, {
        v: 999505344804856076727628n
      })
    })

    it('should withdraw protocol fee', async () => {
      const feeTier = newFeeTier({ v: 10000000000n }, 1)

      const poolKey = newPoolKey(
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )

      const token0Before = await token0.balanceOf(account, account.address.toString())
      const token1Before = await token1.balanceOf(account, account.address.toString())

      const poolBefore = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      assert.deepEqual(poolBefore.feeProtocolTokenX, 1n)
      assert.deepEqual(poolBefore.feeProtocolTokenY, 0n)

      await invariant.withdrawProtocolFee(account, poolKey)

      const poolAfter = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      assert.deepEqual(poolAfter.feeProtocolTokenX, 0n)
      assert.deepEqual(poolAfter.feeProtocolTokenY, 0n)

      const token0After = await token0.balanceOf(account, account.address.toString())
      const token1After = await token1.balanceOf(account, account.address.toString())
      if (poolKey.tokenX === token0.contract.address.toString()) {
        assert.deepEqual(token0Before + 1, token0After)
        assert.deepEqual(token1Before, token1After)
      } else {
        assert.deepEqual(token0Before, token0After)
        assert.deepEqual(token1Before + 1, token1After)
      }
    })

    it('should change fee receiver', async () => {
      const feeTier = newFeeTier({ v: 10000000000n }, 1)

      const poolKey = newPoolKey(
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )

      await invariant.changeFeeReceiver(account, poolKey, testAccount.address.toString())

      const token0Before = await token0.balanceOf(account, testAccount.address.toString())
      const token1Before = await token1.balanceOf(account, testAccount.address.toString())

      const poolBefore = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      assert.deepEqual(poolBefore.feeProtocolTokenX, 1n)
      assert.deepEqual(poolBefore.feeProtocolTokenY, 0n)

      await invariant.withdrawProtocolFee(testAccount, poolKey)

      const poolAfter = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      assert.deepEqual(poolAfter.feeProtocolTokenX, 0n)
      assert.deepEqual(poolAfter.feeProtocolTokenY, 0n)

      const token0After = await token0.balanceOf(account, testAccount.address.toString())
      const token1After = await token1.balanceOf(account, testAccount.address.toString())
      if (poolKey.tokenX === token0.contract.address.toString()) {
        assert.deepEqual(token0Before + 1, token0After)
        assert.deepEqual(token1Before, token1After)
      } else {
        assert.deepEqual(token0Before, token0After)
        assert.deepEqual(token1Before + 1, token1After)
      }
    })
  })

  describe('positions', async () => {
    const lowerTickIndex = -20n
    const upperTickIndex = 10n
    const feeTier = newFeeTier({ v: 6000000000n }, 10)
    let poolKey = _newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    beforeEach(async () => {
      poolKey = _newPoolKey(
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

      await invariant.createPosition(
        account,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        { v: 1000000000000n },
        pool.sqrtPrice,
        pool.sqrtPrice
      )
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
    it('remove position', async () => {
      {
        await invariant.removePosition(account, 0n)
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
      let tokenX
      let tokenY
      if (token0.contract.address.toString() < token1.contract.address.toString()) {
        tokenX = token0
        tokenY = token1
      } else {
        tokenX = token1
        tokenY = token0
      }

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

        assert.equal(swapperX, 0)
        assert.equal(swapperY, 993)

        const invariantX = await tokenX.balanceOf(account, invariant.contract.address.toString())
        const invariantY = await tokenY.balanceOf(account, invariant.contract.address.toString())

        assert.equal(invariantX, 1500)
        assert.equal(invariantY, 7)

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

        const invariantAfterX = await tokenX.balanceOf(
          account,
          invariant.contract.address.toString()
        )

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
})
