import { Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { assert } from 'chai'
import { InvariantError, Position } from 'math/math.js'
import { InvariantTx } from './schema.js'

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
