import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

const feeTier = newFeeTier(10000000000n, 1n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

describe('get liquidity ticks', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0Address,
      token1Address,
      feeTier,
      1000000000000000000000000n,
      0n
    )

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  })

  it('should get liquidity ticks', async () => {
    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      10n,
      1000000000000000000000000n,
      1000000000000000000000000n
    )

    const result = await invariant.getLiquidityTicks(account, poolKey, 0n)
    assert.equal(result.length, 2)
  })

  it('should get liquidity ticks limit', async function () {
    this.timeout(10000)

    for (let i = 1n; i <= 372n; i++) {
      await invariant.createPosition(
        account,
        poolKey,
        -i,
        i,
        10n,
        1000000000000000000000000n,
        1000000000000000000000000n
      )
    }

    const result = await invariant.getLiquidityTicks(account, poolKey, 0n)

    assert.equal(result.length, 372)
  })

  it('should get liquidity ticks with offset', async () => {
    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      10n,
      1000000000000000000000000n,
      1000000000000000000000000n
    )

    const result1 = await invariant.getLiquidityTicks(account, poolKey, 0n)
    assert.equal(result1.length, 2)

    const result2 = await invariant.getLiquidityTicks(account, poolKey, 1n)
    assert.equal(result2.length, 1)

    assert.equal(result1[1].toString(), result2[0].toString())
  })

  it('should get position ticks with multiple queries', async function () {
    this.timeout(15000)

    for (let i = 1n; i <= 400n; i++) {
      await invariant.createPosition(
        account,
        poolKey,
        -i,
        i,
        10n,
        1000000000000000000000000n,
        1000000000000000000000000n
      )
    }

    const liquidityTicks = await invariant.getLiquidityTicksAmount(account, poolKey)

    const promises = []

    for (let i = 0n; i < liquidityTicks; i += 372n) {
      promises.push(invariant.getLiquidityTicks(account, poolKey, i))
    }

    const result = await Promise.all(promises)

    let ticks = 0

    for (let i = 0; i < result.length; i++) {
      ticks += result[i].length
    }

    assert.equal(ticks, 800)
  })
})
