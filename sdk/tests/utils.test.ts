import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { toPercentage, toSqrtPrice } from 'math/math.js'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import {
  calculatePriceImpact,
  calculateSqrtPriceAfterSlippage,
  initPolkadotApi,
  newFeeTier,
  newPoolKey,
  simulateUnclaimedFees
} from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

const invariant = await Invariant.deploy(api, Network.Local, account, { v: 10000000000n })
const token0 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)
const token1 = await PSP22.deploy(api, Network.Local, account, 1000000000n, 'Coin', 'COIN', 0n)

const feeTier = newFeeTier({ v: 10000000000n }, 1n)

describe('utils', () => {
  describe('test calculatePriceImpact', () => {
    it('increasing price', () => {
      // price change       120 -> 599
      // real price impact  79.96661101836...%
      const startingSqrtPrice = { v: 10954451150103322269139395n }
      const endingSqrtPrice = { v: 24474476501040834315678144n }
      const priceImpact = calculatePriceImpact(startingSqrtPrice, endingSqrtPrice)
      assert.equal(priceImpact.v, 799666110183n)
    })

    it('decreasing price', () => {
      // price change       0.367 -> 1.0001^(-221818)
      // real price impact  99.9999999365...%
      const startingSqrtPrice = { v: 605805249234438377196232n }
      const endingSqrtPrice = { v: 15258932449895975601n }
      const priceImpact = calculatePriceImpact(startingSqrtPrice, endingSqrtPrice)
      assert.equal(priceImpact.v, 999999999365n)
    })
  })

  describe('test simulateUnclaimedFees', () => {
    it('should return correct price', async () => {
      await invariant.addFeeTier(account, feeTier)

      await invariant.createPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier,
        { v: 1000000000000000000000000n },
        0n
      )

      await token0.approve(account, invariant.contract.address.toString(), 10000000000000n)
      await token1.approve(account, invariant.contract.address.toString(), 10000000000000n)

      const poolKey = newPoolKey(
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )

      await invariant.createPosition(
        account,
        poolKey,
        -10n,
        10n,
        { v: 10000000000000n },
        { v: 1000000000000000000000000n },
        { v: 1000000000000000000000000n }
      )

      await token0.approve(account, invariant.contract.address.toString(), 1000000000n)
      await token1.approve(account, invariant.contract.address.toString(), 1000000000n)

      await invariant.swap(account, poolKey, true, 4999n, true, {
        v: 999505344804856076727628n
      })

      const pool = await invariant.getPool(
        account,
        token0.contract.address.toString(),
        token1.contract.address.toString(),
        feeTier
      )
      const position = await invariant.getPosition(account, account.address, 0n)
      const lowerTick = await invariant.getTick(account, poolKey, -10n)
      const upperTick = await invariant.getTick(account, poolKey, 10n)

      const result = simulateUnclaimedFees(pool, position, lowerTick, upperTick)

      const token0Before = await token0.balanceOf(account, account.address.toString())
      const token1Before = await token1.balanceOf(account, account.address.toString())

      await invariant.claimFee(account, 0n)

      const token0After = await token0.balanceOf(account, account.address.toString())
      const token1After = await token1.balanceOf(account, account.address.toString())

      if (poolKey.tokenX === token0.contract.address.toString()) {
        assert.equal(token0Before + result.x, token0After)
        assert.equal(token1Before, token1After)
      } else {
        assert.equal(token0Before, token0After)
        assert.equal(token1Before + result.x, token1After)
      }
    })
  })

  describe('test calculateSqrtPriceAfterSlippage', () => {
    it('no slippage up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(0n, 0n)

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, true)

      assert.equal(limitSqrt.v, sqrtPrice)
    })

    it('no slippage down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(0n, 0n)

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, false)

      assert.equal(limitSqrt.v, sqrtPrice)
    })

    it('slippage of 1% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 2n)

      // sqrt(1) * sqrt(1 + 0.01) = 1.0049876
      const expected = { v: 1004987562112089027021926n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, true)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 1% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 2n)

      // sqrt(1) * sqrt(1 - 0.01) = 0.99498744
      const expected = { v: 994987437106619954734479n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, false)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.5% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(5n, 3n)

      // sqrt(1) * sqrt(1 - 0.005) = 1.00249688
      const expected = { v: 1002496882788171067537936n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, true)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.5% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(5n, 3n)

      // sqrt(1) * sqrt(1 - 0.005) = 0.997496867
      const expected = { v: 997496867163000166582694n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, false)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.00003% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(3n, 7n)

      // sqrt(1) * sqrt(1 + 0.0000003) = 1.00000015
      const expected = { v: 1000000149999988750001687n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, true)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 0.00003% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(3n, 7n)

      // sqrt(1) * sqrt(1 - 0.0000003) = 0.99999985
      const expected = { v: 999999849999988749998312n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, false)

      assert.equal(limitSqrt.v, expected.v)
    })

    it('slippage of 100% up', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 0n)

      // sqrt(1) * sqrt(1 + 1) = 1.414213562373095048801688...
      const expected = { v: 1414213562373095048801688n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, true)

      assert.deepEqual(limitSqrt, expected)
    })

    it('slippage of 100% down', () => {
      const sqrtPrice = toSqrtPrice(1n, 0n)
      const slippage = toPercentage(1n, 0n)

      // sqrt(1) * sqrt(1 - 1) = 0
      const expected = { v: 0n }

      const limitSqrt = calculateSqrtPriceAfterSlippage({ v: sqrtPrice }, { v: slippage }, false)

      assert.deepEqual(limitSqrt, expected)
    })
  })
})
