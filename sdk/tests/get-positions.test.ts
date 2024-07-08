import { Pool } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { assert } from 'chai'
import { objectEquals } from '../src/testUtils'
import { describe, it } from 'mocha'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

const feeTier = newFeeTier(6000000000n, 10n)

let poolKey = newPoolKey(token0Address, token1Address, feeTier)
let pool: Pool

describe('get-positions', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n, token1Address)

    pool = await invariant.getPool(token0Address, token1Address, feeTier)

    await invariant.createPosition(account, poolKey, -10n, 10n, 1000000000000n, pool.sqrtPrice, 0n)
    await invariant.createPosition(account, poolKey, -20n, 20n, 1000000000000n, pool.sqrtPrice, 0n)
  })

  it('get positions', async () => {
    const result = await invariant.getPositions(account.address, 2n, 0n)

    assert.equal(result[0].length, 2)
    assert.equal(result[1], 2n)

    const firstExpectedPosition = {
      poolKey,
      liquidity: 1000000000000n,
      lowerTickIndex: -10n,
      upperTickIndex: 10n,
      feeGrowthInsideX: 0n,
      feeGrowthInsideY: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    const firstExpectedPool = {
      liquidity: 2000000000000n,
      sqrtPrice: 1000000000000000000000000n,
      currentTickIndex: 0n,
      feeGrowthGlobalX: 0n,
      feeGrowthGlobalY: 0n,
      feeProtocolTokenX: 0n,
      feeProtocolTokenY: 0n,
      feeReceiver: account.address
    }

    objectEquals(result[0][0][0], firstExpectedPosition, ['lastBlockNumber'])
    objectEquals(result[0][0][1], firstExpectedPool, ['startTimestamp', 'lastTimestamp'])

    const secondExpectedPosition = {
      poolKey,
      liquidity: 1000000000000n,
      lowerTickIndex: -20n,
      upperTickIndex: 20n,
      feeGrowthInsideX: 0n,
      feeGrowthInsideY: 0n,
      tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    const secondExpectedPool = {
      liquidity: 2000000000000n,
      sqrtPrice: 1000000000000000000000000n,
      currentTickIndex: 0n,
      feeGrowthGlobalX: 0n,
      feeGrowthGlobalY: 0n,
      feeProtocolTokenX: 0n,
      feeProtocolTokenY: 0n,
      feeReceiver: account.address
    }

    objectEquals(result[0][1][0], secondExpectedPosition, ['lastBlockNumber'])
    objectEquals(result[0][1][1], secondExpectedPool, ['startTimestamp', 'lastTimestamp'])
  })

  it('get positions less than exist', async () => {
    const result = await invariant.getPositions(account.address, 1n, 0n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })

  it('get positions more than exist', async () => {
    const result = await invariant.getPositions(account.address, 3n, 0n)

    assert.equal(result[0].length, 2)
    assert.equal(result[1], 2n)
  })

  it('get positions with offset', async () => {
    const result = await invariant.getPositions(account.address, 1n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })

  it('get positions with offset less than exist', async () => {
    await invariant.createPosition(account, poolKey, -30n, 30n, 1000000000000n, pool.sqrtPrice, 0n)
    const result = await invariant.getPositions(account.address, 1n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 3n)
  })

  it('get positions with offset more than exist', async () => {
    const result = await invariant.getPositions(account.address, 2n, 1n)

    assert.equal(result[0].length, 1)
    assert.equal(result[1], 2n)
  })
})
