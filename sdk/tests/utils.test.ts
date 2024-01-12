import { assert } from 'chai'
import { calculatePriceImpact } from '../src/utils'

describe('utils', () => {
  describe('test calculatePriceImpact', () => {
    it('increasing price', () => {
      // price change       120 -> 599
      // real price impact  79.96661101836...%
      const startingSqrtPrice = { v: 10954451150103322269139395n }
      const endingSqrtPrice = { v: 24474476501040834315678144n }
      const priceImpact = calculatePriceImpact(startingSqrtPrice, endingSqrtPrice)
      assert.equal(priceImpact.v as bigint, 799666110184n)
    })

    it('decreasing price', () => {
      // price change       0.367 -> 1.0001^(-221818)
      // real price impact  99.9999999365...%
      const startingSqrtPrice = { v: 605805249234438377196232n }
      const endingSqrtPrice = { v: 15258932449895975601n }
      const priceImpact = calculatePriceImpact(startingSqrtPrice, endingSqrtPrice)
      assert.equal(priceImpact.v as bigint, 999999999366n)
    })
  })
})
