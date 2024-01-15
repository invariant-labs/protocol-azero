import { Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { assert } from 'chai'
import {
  CreatePositionEvent,
  InvariantError,
  Percentage,
  Position,
  RemovePositionEvent,
  SqrtPrice
} from 'math/math.js'
import { InvariantTx } from './schema.js'
import { DENOMINATOR, PRICE_DENOMINATOR } from './utils.js'

export const positionEquals = async (recievedPosition: Position, expectedPosition: Position) => {
  assert.deepEqual(recievedPosition.poolKey, expectedPosition.poolKey)
  assert.deepEqual(recievedPosition.liquidity.v, expectedPosition.liquidity.v)
  assert.deepEqual(recievedPosition.lowerTickIndex, expectedPosition.lowerTickIndex)
  assert.deepEqual(recievedPosition.upperTickIndex, expectedPosition.upperTickIndex)
  assert.deepEqual(recievedPosition.feeGrowthInsideX.v, expectedPosition.feeGrowthInsideX.v)
  assert.deepEqual(recievedPosition.feeGrowthInsideY.v, expectedPosition.feeGrowthInsideY.v)
  assert.deepEqual(recievedPosition.tokensOwedX, expectedPosition.tokensOwedX)
  assert.deepEqual(recievedPosition.tokensOwedY, expectedPosition.tokensOwedY)
}

export const assertThrowsAsync = async (fn: Promise<any>, word?: InvariantError | InvariantTx) => {
  try {
    await fn
  } catch (e: any) {
    if (word) {
      const err = e.toString()
      console.log(err)
      const regex = new RegExp(`${word}$`)
      if (!regex.test(err)) {
        console.log(err)
        throw new Error('Invalid Error message')
      }
    }
    return
  }
  throw new Error('Function did not throw error')
}

export const sleep = async (ms: number) => {
  return await new Promise(resolve => setTimeout(resolve, ms))
}

export const getEnvTestAccount = async (keyring: Keyring): Promise<IKeyringPair> => {
  const accountUri = process.env.TEST_ACCOUNT_URI

  if (!accountUri) {
    throw new Error('invalid account uri')
  }

  return keyring.addFromUri(accountUri)
}

export const toSqrtPrice = (x: bigint, decimals: bigint = 0n): SqrtPrice => {
  return toDecimalWithDenominator(x, PRICE_DENOMINATOR, decimals)
}

export const toPercentage = (x: bigint, decimals: bigint = 0n): Percentage => {
  return toDecimalWithDenominator(x, DENOMINATOR, decimals)
}

export const toDecimalWithDenominator = (x: bigint, denominator: bigint, decimals: bigint = 0n) => {
  return { v: (denominator * x) / BigInt(Math.pow(10, Number(decimals))) }
}

export const createPositionEventEquals = (
  createPositionEvent: CreatePositionEvent,
  expectedCreatePositionEvent: CreatePositionEvent
) => {
  assert.deepEqual(createPositionEvent.address, expectedCreatePositionEvent.address)
  assert.deepEqual(
    createPositionEvent.currentSqrtPrice,
    expectedCreatePositionEvent.currentSqrtPrice
  )
  assert.deepEqual(createPositionEvent.liquidity, expectedCreatePositionEvent.liquidity)
  assert.deepEqual(createPositionEvent.lowerTick, expectedCreatePositionEvent.lowerTick)
  assert.deepEqual(createPositionEvent.pool, expectedCreatePositionEvent.pool)
  assert.deepEqual(createPositionEvent.upperTick, expectedCreatePositionEvent.upperTick)
}

export const removePositionEventEquals = (
  removePositionEvent: RemovePositionEvent,
  expectedRemovePositionEvent: RemovePositionEvent
) => {
  assert.deepEqual(removePositionEvent.address, expectedRemovePositionEvent.address)
  assert.deepEqual(
    removePositionEvent.currentSqrtPrice,
    expectedRemovePositionEvent.currentSqrtPrice
  )
  assert.deepEqual(removePositionEvent.liquidity, expectedRemovePositionEvent.liquidity)
  assert.deepEqual(removePositionEvent.lowerTick, expectedRemovePositionEvent.lowerTick)
  assert.deepEqual(removePositionEvent.pool, expectedRemovePositionEvent.pool)
  assert.deepEqual(removePositionEvent.upperTick, expectedRemovePositionEvent.upperTick)
}
