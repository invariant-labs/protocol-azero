import { assert } from 'chai'
import { SqrtPrice } from 'clamm/math.js'
describe('wasm', async () => {
  const clammSqrtPrice: SqrtPrice = 0n
  assert.equal(clammSqrtPrice, 0n)
})
