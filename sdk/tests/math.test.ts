import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import {
  Position,
  SqrtPrice,
  getLiquidityByX,
  getLiquidityByY,
  isTokenX
} from 'invariant-a0-wasm/invariant_a0_wasm.js'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { assertThrowsAsync, objectEquals } from '../src/testUtils'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

describe('get-liquidity-by-x', async () => {
  const providedAmount = 430000n

  const feeTier = newFeeTier(6000000000n, 10n)
  const positionOwner = keyring.addFromUri('//Bob')

  let poolKey = newPoolKey(token0Address, token1Address, feeTier)

  let [tokenX, tokenY] = isTokenX(token0Address, token1Address)
    ? [token0Address, token1Address]
    : [token1Address, token0Address]

  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    if (isTokenX(token0Address, token1Address)) {
      tokenX = token0Address
      tokenY = token1Address
    } else {
      tokenX = token1Address
      tokenY = token0Address
    }

    await invariant.addFeeTier(account, feeTier)

    const initSqrtPrice: SqrtPrice = 1005012269622000000000000n
    await invariant.createPool(account, poolKey, initSqrtPrice)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  })
  it('check get liquidity by x', async () => {
    // below range
    {
      const lowerTickIndex = 80n
      const upperTickIndex = 120n

      const pool = await invariant.getPool(account.address, token0Address, token1Address, feeTier)

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

      const pool = await invariant.getPool(account.address, token0Address, token1Address, feeTier)

      const { l, amount } = getLiquidityByX(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      await psp22.setContractAddress(tokenX)
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)
      await psp22.setContractAddress(tokenY)
      await psp22.mint(positionOwner, amount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), amount)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        0n
      )

      const position = await invariant.getPosition(positionOwner.address, positionOwner.address, 0n)
      const expectedPosition: Position = {
        poolKey: poolKey,
        liquidity: l,
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
    // above range
    {
      const lowerTickIndex = 150n
      const upperTickIndex = 800n

      const pool = await invariant.getPool(account.address, token0Address, token1Address, feeTier)

      const { l, amount } = getLiquidityByX(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      assert.deepEqual(amount, 0n)

      await psp22.setContractAddress(tokenX)
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        0n
      )

      const position = await invariant.getPosition(positionOwner.address, positionOwner.address, 1n)
      const expectedPosition: Position = {
        poolKey: poolKey,
        liquidity: l,
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
})

describe('get-liquidity-by-y', async () => {
  const providedAmount = 47600000000n
  const feeTier = newFeeTier(6000000000n, 10n)
  const positionOwner = keyring.addFromUri('//Bob')

  let poolKey = newPoolKey(token0Address, token1Address, feeTier)
  let [tokenX, tokenY] = isTokenX(token0Address, token1Address)
    ? [token0Address, token1Address]
    : [token1Address, token0Address]
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    if (isTokenX(token0Address, token1Address)) {
      tokenX = token0Address
      tokenY = token1Address
    } else {
      tokenX = token1Address
      tokenY = token0Address
    }

    await invariant.addFeeTier(account, feeTier)

    const initSqrtPrice: SqrtPrice = 367897834491000000000000n

    await invariant.createPool(account, poolKey, initSqrtPrice)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  })
  it('check get liquidity by y', async () => {
    // below range
    {
      const lowerTickIndex = -22000n
      const upperTickIndex = -21000n

      const pool = await invariant.getPool(account.address, token0Address, token1Address, feeTier)

      const { l, amount } = getLiquidityByY(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      assert.deepEqual(amount, 0n)

      await psp22.setContractAddress(tokenY)
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        0n
      )

      const position = await invariant.getPosition(positionOwner.address, positionOwner.address, 0n)
      const expectedPosition: Position = {
        poolKey: poolKey,
        liquidity: l,
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
    // in range
    {
      const lowerTickIndex = -25000n
      const upperTickIndex = -19000n

      const pool = await invariant.getPool(account.address, token0Address, token1Address, feeTier)

      const { l, amount } = getLiquidityByY(
        providedAmount,
        lowerTickIndex,
        upperTickIndex,
        pool.sqrtPrice,
        true
      )

      await psp22.setContractAddress(tokenY)
      await psp22.mint(positionOwner, providedAmount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), providedAmount)
      await psp22.setContractAddress(tokenX)
      await psp22.mint(positionOwner, amount)
      await psp22.approve(positionOwner, invariant.contract.address.toString(), amount)

      await invariant.createPosition(
        positionOwner,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        l,
        pool.sqrtPrice,
        0n
      )

      const position = await invariant.getPosition(positionOwner.address, positionOwner.address, 1n)
      const expectedPosition: Position = {
        poolKey: poolKey,
        liquidity: l,
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
    // above range
    {
      const lowerTickIndex = -10000n
      const upperTickIndex = 0n

      const pool = await invariant.getPool(account.address, token0Address, token1Address, feeTier)

      assertThrowsAsync(
        new Promise(() => {
          getLiquidityByY(providedAmount, lowerTickIndex, upperTickIndex, pool.sqrtPrice, true)
        })
      )
    }
  })
})
