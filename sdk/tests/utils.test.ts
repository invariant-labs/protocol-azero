import { assert } from 'chai'
import { toPercentage, toSqrtPrice } from '../src/testUtils'
import { calculateSqrtPriceAfterSlippage } from '../src/utils'

describe('utils', () => {
  describe('test calculateSqrtPriceAfterSlippage', () => {
    it('no slippage up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(0n)

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt.v, sqrtPrice.v)
    })

    it('no slippage down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(0n)

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt.v, sqrtPrice.v)
    })

    it('slippage of 1% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n, 2n)

      // sqrt(1) * sqrt(1 + 0.01) = 1.0049876
      const expected = { v: 1004987562112000000000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 1% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n, 2n)

      // sqrt(1) * sqrt(1 - 0.01) = 0.99498744
      const expected = { v: 994987437107000000000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.5% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(5n, 3n)

      // sqrt(1) * sqrt(1 - 0.005) = 1.00249688
      const expected = { v: 1002496882788000000000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.5% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(5n, 3n)

      // sqrt(1) * sqrt(1 - 0.005) = 0.997496867
      const expected = { v: 997496867163000000000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.00003% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(3n, 7n)

      // sqrt(1) * sqrt(1 + 0.0000003) = 1.00000015
      const expected = { v: 1000000150000000000000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.00003% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(3n, 7n)

      // sqrt(1) * sqrt(1 - 0.0000003) = 0.99999985
      const expected = { v: 999999850000000000000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 100% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n)

      // sqrt(1) * sqrt(1 + 1) = 1.414213562373...
      const expected = { v: 1414213562373000000000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.deepEqual(limitSqrt, expected)
    })

    it('slippage of 100% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n)

      // sqrt(1) * sqrt(1 - 1) = 0
      const expected = { v: 0n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.deepEqual(limitSqrt, expected)
    })
  })
})
