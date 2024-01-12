import { assert } from 'chai'
import { toPercentage, toSqrtPrice } from '../src/testUtils'
import { PRICE_DENOMINATOR, calculatePriceAfterSlippage } from '../src/utils'

describe('utils', () => {
  describe('test calculatePriceAfterSlippage', () => {
    it('no slippage up', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(0n)

      const expected = PRICE_DENOMINATOR

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected)
    })

    it('no slippage down', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(0n)

      const expected = PRICE_DENOMINATOR

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected)
    })

    it('slippage of 1% up', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(1n, 2n)

      const expected = { v: 1009999999999821040955554n }

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected.v)
    })

    it('slippage of 1% down', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(1n, 2n)

      const expected = { v: 990000000000756263920003n }

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.5% up', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(5n, 3n)

      const expected = { v: 1004999999999656993791841n }

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.5% down', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(5n, 3n)

      const expected = { v: 994999999999999650975237n }

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.00001% up', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(3n, 7n)

      const expected = { v: 1000000300000022483222777n }

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected.v)
    })

    it('slippage of 0.00001% down', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(3n, 7n)

      const expected = { v: 999999700000022483222787n }

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected.v)
    })

    it('slippage of 100% up', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(1n)

      const expected = { v: 1999999999999731127836695n }

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, true)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected.v)
    })

    it('slippage of 100% down', async () => {
      const price = toSqrtPrice(1n)
      const slippage = toPercentage(1n)

      const expected = 0n

      const limitSqrt = calculatePriceAfterSlippage(price, slippage, false)
      const limit = ((limitSqrt.v as bigint) * (limitSqrt.v as bigint)) / PRICE_DENOMINATOR

      assert.equal(limit, expected)
    })
  })
})
