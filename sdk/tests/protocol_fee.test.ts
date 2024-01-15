import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { InvariantError } from 'math/math.js'
import { Network } from '../src/network'
import { assertThrowsAsync } from '../src/testUtils'
import { deployInvariant, deployPSP22, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const testAccount = await keyring.addFromUri('//Bob')

let invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

const feeTier = newFeeTier({ v: 10000000000n }, 1n)

describe('protocol_fee', async () => {
  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n }, Network.Local)
    token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)
    token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n, Network.Local)

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
  })

  it('should withdraw protocol fee', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 1n)

    const poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    const token0Before = await token0.balanceOf(account, account.address.toString())
    const token1Before = await token1.balanceOf(account, account.address.toString())

    const poolBefore = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    assert.deepEqual(poolBefore.feeProtocolTokenX, 1n)
    assert.deepEqual(poolBefore.feeProtocolTokenY, 0n)

    await invariant.withdrawProtocolFee(account, poolKey)

    const poolAfter = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    assert.deepEqual(poolAfter.feeProtocolTokenX, 0n)
    assert.deepEqual(poolAfter.feeProtocolTokenY, 0n)

    const token0After = await token0.balanceOf(account, account.address.toString())
    const token1After = await token1.balanceOf(account, account.address.toString())
    if (poolKey.tokenX === token0.contract.address.toString()) {
      assert.deepEqual(token0Before + 1n, token0After)
      assert.deepEqual(token1Before, token1After)
    } else {
      assert.deepEqual(token0Before, token0After)
      assert.deepEqual(token1Before + 1n, token1After)
    }
  })

  it('should change fee receiver', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 1n)

    const poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    await invariant.changeFeeReceiver(account, poolKey, testAccount.address.toString())

    const token0Before = await token0.balanceOf(account, testAccount.address.toString())
    const token1Before = await token1.balanceOf(account, testAccount.address.toString())

    const poolBefore = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    assert.deepEqual(poolBefore.feeProtocolTokenX, 1n)
    assert.deepEqual(poolBefore.feeProtocolTokenY, 0n)

    await invariant.withdrawProtocolFee(testAccount, poolKey)
    assertThrowsAsync(
      invariant.withdrawProtocolFee(account, poolKey),
      InvariantError.NotFeeReceiver
    )

    const poolAfter = await invariant.getPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )
    assert.deepEqual(poolAfter.feeProtocolTokenX, 0n)
    assert.deepEqual(poolAfter.feeProtocolTokenY, 0n)

    const token0After = await token0.balanceOf(account, testAccount.address.toString())
    const token1After = await token1.balanceOf(account, testAccount.address.toString())
    if (poolKey.tokenX === token0.contract.address.toString()) {
      assert.deepEqual(token0Before + 1n, token0After)
      assert.deepEqual(token1Before, token1After)
    } else {
      assert.deepEqual(token0Before, token0After)
      assert.deepEqual(token1Before + 1n, token1After)
    }
  })
})
