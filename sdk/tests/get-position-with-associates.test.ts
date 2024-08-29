import {
  CreatePositionEvent,
  InvariantError,
  Pool
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { assertThrowsAsync, objectEquals } from '../src/testUtils'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { describe, it } from 'mocha'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

const lowerTickIndex = -20n
const upperTickIndex = 10n
const feeTier = newFeeTier(6000000000n, 10n)

let poolKey = newPoolKey(token0Address, token1Address, feeTier)
let pool: Pool

describe('get-position-with-associates', async () => {
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

    const result = await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      1000000000000n,
      pool.sqrtPrice,
      0n
    )

    const expectedCreatePositionEvent: CreatePositionEvent = {
      address: account.address.toString(),
      currentSqrtPrice: 1000000000000000000000000n,
      liquidity: 1000000000000n,
      lowerTick: -20n,
      pool: poolKey,
      upperTick: 10n,
      timestamp: 0n
    }

    objectEquals(result.events[4], expectedCreatePositionEvent, ['timestamp'])
  })

  it('position, pool and ticks match', async () => {
    const positionRegular = await invariant.getPosition(account.address, 0n)
    const poolRegular = await invariant.getPool(token0Address, token1Address, poolKey.feeTier)
    const lowerTickRegular = await invariant.getTick(poolKey, positionRegular.lowerTickIndex)
    const upperTickRegular = await invariant.getTick(poolKey, positionRegular.upperTickIndex)

    const [position, pool, lowerTick, upperTick] = await invariant.getPositionWithAssociates(
      account.address,
      0n
    )

    assert.deepEqual(position, positionRegular)
    assert.deepEqual(pool, poolRegular)
    assert.deepEqual(lowerTick, lowerTickRegular)
    assert.deepEqual(upperTick, upperTickRegular)
  })

  it('position does not exist', async () => {
    await assertThrowsAsync(
      invariant.getPositionWithAssociates(account.address, 1n),
      InvariantError.PositionNotFound
    )
  })
})
