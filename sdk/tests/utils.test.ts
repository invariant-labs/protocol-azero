import { assert } from 'chai'
import { getSqrtPriceDenominator } from 'math/math.js'
import { toPercentage, toSqrtPrice } from '../src/testUtils'
import { calculateSqrtPriceAfterSlippage } from '../src/utils'

describe('utils', () => {
  describe('test calculatePriceAfterSlippage', () => {
    it('no slippage up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(0n)

      const expected = getSqrtPriceDenominator()

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected)
    })

    it('no slippage down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(0n)

      const expected = getSqrtPriceDenominator()

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected)
    })

    it('slippage of 1% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n, 2n)

      const expected = { v: 1009999999999821057900544n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected.v)
    })

    it('slippage of 1% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n, 2n)

      const expected = { v: 990000000000756280529449n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.5% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(5n, 3n)

      const expected = { v: 1004999999999657010652944n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.5% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(5n, 3n)

      const expected = { v: 994999999999999667668569n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.00001% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(3n, 7n)

      const expected = { v: 1000000300000022500000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.00001% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(3n, 7n)

      const expected = { v: 999999700000022500000000n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected.v)
    })

    it('slippage of 100% up', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n)

      const expected = { v: 1999999999999731161391129n }

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected.v)
    })

    it('slippage of 100% down', () => {
      const sqrtPrice = toSqrtPrice(1n)
      const slippage = toPercentage(1n)

      const expected = 0n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / getSqrtPriceDenominator()

      assert.equal(limit, expected)
    })
  })
})
