import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { newFeeTier, newPoolKey } from 'math/math.js'
import { Network } from '../src/network'
import { deployInvariant, deployPSP22, initPolkadotApi, simulateUnclaimedFees } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

const invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
const token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
const token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

const feeTier = newFeeTier({ v: 10000000000n }, 1)

describe('utils', () => {
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
})
