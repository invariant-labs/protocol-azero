import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Position, SqrtPrice, getLiquidityByX, getLiquidityByY } from 'math/math.js'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { assertThrowsAsync, positionEquals } from '../src/testUtils'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, { v: 10000000000n })
let token0 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = token0

describe('check get liquidity by x', async () => {
  const providedAmount = 430000n

  const feeTier = newFeeTier({ v: 6000000000n }, 10n)
  const positionOwner = keyring.addFromUri('//Bob')

  let poolKey = newPoolKey(
    token0.contract.address.toString(),
    token1.contract.address.toString(),
    feeTier
  )
  let tokenX = token0
  let tokenY = token1

  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, { v: 10000000000n })
    token0 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(
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

    await psp22.setContractAddress(token0.contract.address.toString())
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1.contract.address.toString())
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
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

      await psp22.setContractAddress(tokenX.contract.address.toString())
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)
      await psp22.setContractAddress(tokenY.contract.address.toString())
      await psp22.mint(positionOwner, amount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), amount)

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

      await psp22.setContractAddress(tokenX.contract.address.toString())
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)

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
  const feeTier = newFeeTier({ v: 6000000000n }, 10n)
  const positionOwner = keyring.addFromUri('//Bob')

  let poolKey = newPoolKey(
    token0.contract.address.toString(),
    token1.contract.address.toString(),
    feeTier
  )
  let tokenX = token0
  let tokenY = token1

  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, { v: 10000000000n })
    token0 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(
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

    await psp22.setContractAddress(token0.contract.address.toString())
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1.contract.address.toString())
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
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

      await psp22.setContractAddress(tokenY.contract.address.toString())
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)

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

      await psp22.setContractAddress(tokenY.contract.address.toString())
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)
      await psp22.setContractAddress(tokenX.contract.address.toString())
      await psp22.mint(positionOwner, amount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), amount)

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
