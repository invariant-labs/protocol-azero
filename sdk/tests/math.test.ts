import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Position, SqrtPrice, newFeeTier } from 'math/math.js'
import { Network } from '../src/network'
import { assertThrowsAsync, positionEquals } from '../src/testUtils'
import {
  _newPoolKey,
  deployInvariant,
  deployPSP22,
  getLiquidityByX,
  getLiquidityByY,
  initPolkadotApi
} from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

describe('invariant', async () => {
  const feeTier = newFeeTier({ v: 10000000000n }, 1)

  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
    token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
  })

  describe('check get liquidity by x', async () => {
    const providedAmount = 430000n
    const feeTier = newFeeTier({ v: 6000000000n }, 10)
    const positionOwner = keyring.addFromUri('//Bob')

    let poolKey = _newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    let tokenX = token0
    let tokenY = token1

    beforeEach(async () => {
      poolKey = _newPoolKey(
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )

      if (token0.contract.address.toString() < token1.contract.address.toString()) {
        tokenX = token0
        tokenY = token1
      } else {
        tokenX = token1
        tokenY = token0
      }

      await invariant.addFeeTier(account, feeTier)

      const initSqrtPrice: SqrtPrice = { v: 1005012269622000000000000n }
      await invariant.createPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier,
        initSqrtPrice,
        100n
      )

      await token0.approve(account, invariant.contract.address.toString(), 10000000000n)
      await token1.approve(account, invariant.contract.address.toString(), 10000000000n)
    })
    it('check get liquidity by x', async () => {
      // below range
      {
        const lowerTickIndex = 80n
        const upperTickIndex = 120n

        const pool = await invariant.getPool(
          account,
          token0.contract.address.toString(),
          token1.contract.address.toString(),
          feeTier
        )

        assertThrowsAsync(
          new Promise(() => {
            getLiquidityByX(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
          })
        )
      }
      // in range
      {
        const lowerTickIndex = 80n
        const upperTickIndex = 120n

        const pool = await invariant.getPool(
          account,
          token0.contract.address.toString(),
          token1.contract.address.toString(),
          feeTier
        )

        const { l, amount } = getLiquidityByX(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        await tokenX.mint(positionOwner, providedAmount)
        await tokenX.approve(positionOwner, invariant.contract.address.toString(), providedAmount)
        await tokenY.mint(positionOwner, amount)
        await tokenY.approve(positionOwner, invariant.contract.address.toString(), amount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          pool.sqrtPrice
        )

        const position = await invariant.getPosition(positionOwner, positionOwner.address, 0n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
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
      // above range
      {
        const lowerTickIndex = 150n
        const upperTickIndex = 800n

        const pool = await invariant.getPool(
          account,
          token0.contract.address.toString(),
          token1.contract.address.toString(),
          feeTier
        )

        const { l, amount } = getLiquidityByX(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        assert.deepEqual(amount, 0n)

        await tokenX.mint(positionOwner, providedAmount)
        await tokenX.approve(positionOwner, invariant.contract.address.toString(), providedAmount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          pool.sqrtPrice
        )

        const position = await invariant.getPosition(positionOwner, positionOwner.address, 1n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
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
  })

  describe('check get liquidity by y', async () => {
    const providedAmount = 47600000000n
    const feeTier = newFeeTier({ v: 6000000000n }, 10)
    const positionOwner = keyring.addFromUri('//Bob')

    let poolKey = _newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    let tokenX = token0
    let tokenY = token1

    beforeEach(async () => {
      poolKey = _newPoolKey(
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )

      if (token0.contract.address.toString() < token1.contract.address.toString()) {
        tokenX = token0
        tokenY = token1
      } else {
        tokenX = token1
        tokenY = token0
      }

      await invariant.addFeeTier(account, feeTier)

      const initSqrtPrice: SqrtPrice = { v: 367897834491000000000000n }
      await invariant.createPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier,
        initSqrtPrice,
        -20000n
      )

      await token0.approve(account, invariant.contract.address.toString(), 10000000000n)
      await token1.approve(account, invariant.contract.address.toString(), 10000000000n)
    })
    it('check get liquidity by y', async () => {
      // below range
      {
        const lowerTickIndex = -22000n
        const upperTickIndex = -21000n

        const pool = await invariant.getPool(
          account,
          token0.contract.address.toString(),
          token1.contract.address.toString(),
          feeTier
        )

        const { l, amount } = getLiquidityByY(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        assert.deepEqual(amount, 0n)

        await tokenY.mint(positionOwner, providedAmount)
        await tokenY.approve(positionOwner, invariant.contract.address.toString(), providedAmount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          pool.sqrtPrice
        )

        const position = await invariant.getPosition(positionOwner, positionOwner.address, 0n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
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
      // in range
      {
        const lowerTickIndex = -25000n
        const upperTickIndex = -19000n

        const pool = await invariant.getPool(
          account,
          token0.contract.address.toString(),
          token1.contract.address.toString(),
          feeTier
        )

        const { l, amount } = getLiquidityByY(
          providedAmount,
          lowerTickIndex,
          upperTickIndex,
          pool.sqrtPrice,
          true
        )

        await tokenY.mint(positionOwner, providedAmount)
        await tokenY.approve(positionOwner, invariant.contract.address.toString(), providedAmount)
        await tokenX.mint(positionOwner, amount)
        await tokenX.approve(positionOwner, invariant.contract.address.toString(), amount)

        await invariant.createPosition(
          positionOwner,
          poolKey,
          lowerTickIndex,
          upperTickIndex,
          l,
          pool.sqrtPrice,
          pool.sqrtPrice
        )

        const position = await invariant.getPosition(positionOwner, positionOwner.address, 1n)
        const expectedPosition: Position = {
          poolKey: poolKey,
          liquidity: l,
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
      // above range
      {
        const lowerTickIndex = -10000n
        const upperTickIndex = 0n

        const pool = await invariant.getPool(
          account,
          token0.contract.address.toString(),
          token1.contract.address.toString(),
          feeTier
        )

        assertThrowsAsync(
          new Promise(() => {
            getLiquidityByY(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
          })
        )
      }
    })
  })
})
