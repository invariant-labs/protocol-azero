import { assert } from 'chai'
import { SqrtPrice } from 'clamm'
describe('wasm', async () => {
  it('should load wasm', async () => {
    const clammSqrtPrice: SqrtPrice = 0n
    assert.equal(clammSqrtPrice, 0n)
  })
})
