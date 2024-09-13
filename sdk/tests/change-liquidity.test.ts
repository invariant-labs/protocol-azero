import {
  Pool,
  Position,
} from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'
import { Keyring } from '@polkadot/api'
import { describe, it } from 'mocha'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { objectEquals } from '../src/testUtils'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

const lowerTickIndex = -20n
const upperTickIndex = 10n
const feeTier = newFeeTier(6000000000n, 10n)

let poolKey = newPoolKey(token0Address, token1Address, feeTier)
let pool: Pool

describe('change-liquidity', async () => {
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

    await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      1000000000000n,
      pool.sqrtPrice,
      0n
    )
  })

  it('change liquidity', async () => {
    const pool = await invariant.getPool(poolKey.tokenX,poolKey.tokenY,feeTier)
    {
      const positionBefore = await invariant.getPosition(account.address, 0n);
      await invariant.changeLiquidity(account, 0n, 1000000000000n, true, pool.sqrtPrice, 0n)
      const positionAfter = await invariant.getPosition(account.address, 0n);
      const expectedPosition: Position=  {
        poolKey: positionBefore.poolKey,
        liquidity: 2000000000000n,
        lowerTickIndex: positionBefore.lowerTickIndex,
        upperTickIndex: positionBefore.upperTickIndex,
        feeGrowthInsideX: 0n,
        feeGrowthInsideY: 0n,
        lastBlockNumber: 0n,
        tokensOwedX: 0n,
        tokensOwedY: 0n,
        createdAt: 0n
      };
  
      objectEquals(
        expectedPosition,
        positionAfter,
        ['createdAt', 'lastBlockNumber']
      )
    }
    {
      const positionBefore = await invariant.getPosition(account.address, 0n);
      await invariant.changeLiquidity(account, 0n, 1000000000000n, false, pool.sqrtPrice, 0n)
      const positionAfter = await invariant.getPosition(account.address, 0n);
      const expectedPosition: Position=  {
        poolKey: positionBefore.poolKey,
        liquidity: 1000000000000n,
        lowerTickIndex: positionBefore.lowerTickIndex,
        upperTickIndex: positionBefore.upperTickIndex,
        feeGrowthInsideX: 0n,
        feeGrowthInsideY: 0n,
        lastBlockNumber: 0n,
        tokensOwedX: 0n,
        tokensOwedY: 0n,
        createdAt: 0n
      };
  
      objectEquals(
        expectedPosition,
        positionAfter,
        ['createdAt', 'lastBlockNumber']
      )
    }
  })
})
