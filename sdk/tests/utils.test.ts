import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import {
  calculateFee,
  calculatePriceImpact,
  calculateSqrtPriceAfterSlippage,
  initPolkadotApi,
  newFeeTier,
  newPoolKey,
  priceToSqrtPrice,
  sqrtPriceToPrice
} from '../src/utils'
import { toPercentage, toSqrtPrice } from '../src/wasm/pkg/invariant_a0_wasm'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

const invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

const feeTier = newFeeTier(10000000000n, 1n)

describe('utils', () => {
  describe('test calculatePriceImpact', () => {
    it('increasing price', () => {
      // price change       120 -> 599
      // real price impact  79.96661101836...%
      const startingSqrtPrice = 10954451150103322269139395n
      const endingSqrtPrice = 24474476501040834315678144n
      const priceImpact = calculatePriceImpact(startingSqrtPrice, endingSqrtPrice)
      assert.equal(priceImpact, 799666110183n)
    })

    it('decreasing price', () => {
      // price change       0.367 -> 1.0001^(-221818)
      // real price impact  99.9999999365...%
      const startingSqrtPrice = 605805249234438377196232n
      const endingSqrtPrice = 15258932449895975601n
      const priceImpact = calculatePriceImpact(startingSqrtPrice, endingSqrtPrice)
      assert.equal(priceImpact, 999999999365n)
    })
  })

  describe('test calculateFee', () => {
    beforeEach(async () => {
      token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
      token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    })
    it('should return correct price', async () => {
      await invariant.addFeeTier(account, feeTier)

      const poolKey = newPoolKey(token0Address, token1Address, feeTier)

      await invariant.createPool(account, poolKey, 1000000000000000000000000n)

      await psp22.setContractAddress(token0Address)
      await psp22.approve(account, invariant.contract.address.toString(), 10000000000000n)
      await psp22.setContractAddress(token1Address)
      await psp22.approve(account, invariant.contract.address.toString(), 10000000000000n)

      await invariant.createPosition(
        account,
        poolKey,
        -10n,
        10n,
        10000000000000n,
        1000000000000000000000000n,
        0n
      )

      await psp22.setContractAddress(token0Address)
      await psp22.approve(account, invariant.contract.address.toString(), 1000000000n)
      await psp22.setContractAddress(token1Address)
      await psp22.approve(account, invariant.contract.address.toString(), 1000000000n)

      await invariant.swap(account, poolKey, true, 4999n, true, 999505344804856076727628n)

      const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
      const position = await invariant.getPosition(account, account.address, 0n)
      const lowerTick = await invariant.getTick(account, poolKey, -10n)
      const upperTick = await invariant.getTick(account, poolKey, 10n)

      const result = calculateFee(pool, position, lowerTick, upperTick)

      await psp22.setContractAddress(token0Address)
      const token0Before = await psp22.balanceOf(account, account.address.toString())
      await psp22.setContractAddress(token1Address)
      const token1Before = await psp22.balanceOf(account, account.address.toString())

      await invariant.claimFee(account, 0n)

      await psp22.setContractAddress(token0Address)
      const token0After = await psp22.balanceOf(account, account.address.toString())
      await psp22.setContractAddress(token1Address)
      const token1After = await psp22.balanceOf(account, account.address.toString())

      if (poolKey.tokenX === token0Address) {
        assert.equal(token0Before + result[0], token0After)
        assert.equal(token1Before, token1After)
      } else {
        assert.equal(token0Before, token0After)
        assert.equal(token1Before + result[0], token1After)
      }
    })
  })

  describe('test calculateSqrtPriceAfterSlippage', () => {
    it('no slippage up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(0n, 0n)

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt, sqrtPrice)
    })

    it('no slippage down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(0n, 0n)

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt, sqrtPrice)
    })

    it('slippage of 1% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 2n)

      // sqrt(1) * sqrt(1 + 0.01) = 1.0049876
      const expected = 1004987562112089027021926n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt, expected)
    })

    it('slippage of 1% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 2n)

      // sqrt(1) * sqrt(1 - 0.01) = 0.99498744
      const expected = 994987437106619954734479n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt, expected)
    })

    it('slippage of 0.5% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(5n, 3n)

      // sqrt(1) * sqrt(1 - 0.005) = 1.00249688
      const expected = 1002496882788171067537936n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt, expected)
    })

    it('slippage of 0.5% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(5n, 3n)

      // sqrt(1) * sqrt(1 - 0.005) = 0.997496867
      const expected = 997496867163000166582694n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt, expected)
    })

    it('slippage of 0.00003% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(3n, 7n)

      // sqrt(1) * sqrt(1 + 0.0000003) = 1.00000015
      const expected = 1000000149999988750001687n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.equal(limitSqrt, expected)
    })

    it('slippage of 0.00003% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(3n, 7n)

      // sqrt(1) * sqrt(1 - 0.0000003) = 0.99999985
      const expected = 999999849999988749998312n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.equal(limitSqrt, expected)
    })

    it('slippage of 100% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 0n)

      // sqrt(1) * sqrt(1 + 1) = 1.414213562373095048801688...
      const expected = 1414213562373095048801688n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, true)

      assert.deepEqual(limitSqrt, expected)
    })

    it('slippage of 100% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 0n)

      // sqrt(1) * sqrt(1 - 1) = 0
      const expected = 0n

      const limitSqrt = calculateSqrtPriceAfterSlippage(sqrtPrice, slippage, false)

      assert.deepEqual(limitSqrt, expected)
    })
  })

  describe('sqrt price and price conversion', () => {
    it('price of 1.00', () => {
      // 1.00 = sqrt(1.00)
      const sqrtPrice = priceToSqrtPrice(1000000000000000000000000n)
      const expectedSqrtPrice = 1000000000000000000000000n

      assert.deepEqual(sqrtPrice, expectedSqrtPrice)
    })

    it('price of 2.00', () => {
      // 1.414213562373095048801688... = sqrt(2.00)
      const sqrtPrice = priceToSqrtPrice(2000000000000000000000000n)
      const expectedSqrtPrice = 1414213562373095048801688n

      assert.deepEqual(sqrtPrice, expectedSqrtPrice)
    })

    it('price of 0.25', () => {
      // 0.5 = sqrt(0.25)
      const sqrtPrice = priceToSqrtPrice(250000000000000000000000n)
      const expectedSqrtPrice = 500000000000000000000000n

      assert.deepEqual(sqrtPrice, expectedSqrtPrice)
    })

    it('sqrt price of 1.00', () => {
      // sqrt(1.00) = 1.00
      const price = sqrtPriceToPrice(1000000000000000000000000n)
      const expectedSqrtPrice = 1000000000000000000000000n

      assert.deepEqual(price, expectedSqrtPrice)
    })

    it('sqrt price of 2.00', () => {
      // sqrt(1.414213562373095048801688...) = 2.00
      const price = sqrtPriceToPrice(1414213562373095048801688n)
      const expectedSqrtPrice = 1999999999999999999999997n

      assert.deepEqual(price, expectedSqrtPrice)
    })

    it('sqrt price of 0.25', () => {
      // sqrt(0.25) = 0.5
      const price = sqrtPriceToPrice(500000000000000000000000n)
      const expectedSqrtPrice = 250000000000000000000000n

      assert.deepEqual(price, expectedSqrtPrice)
    })
  })
})
