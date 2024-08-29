import { InvariantError } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { assert } from 'chai'
import { InvariantTx } from './schema.js'

export const objectEquals = (
  object: { [key: string]: any },
  expectedObject: { [key: string]: any },
  keys: string[]
) => {
  for (const key in object) {
    if (!keys.includes(key)) {
      assert.deepEqual(object[key], expectedObject[key], `Key missing: ${key}`)
    }
  }
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
