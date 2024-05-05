import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant, PSP22 } from '../src'
import { Network } from '../src/network'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

const network = Network.Testnet
const api = await initPolkadotApi(network)

// const deployOptions: ContractOptions = {
//   storageDepositLimit: null,
//   refTime: 259058343000,
//   proofSize: 1160117
// }

const keyring = new Keyring({ type: 'sr25519' })
const account = keyring.addFromUri('//Bob')

describe('tickmap', async () => {
  const feeTier = newFeeTier(6000000000n, 1n)
  const ticks = [-221818n, -221817n, -58n, 5n, 221817n, 221818n]
  const invariant = await Invariant.deploy(api, network, account, 10000000000n)
  const token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
  const token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
  const psp22 = await PSP22.load(api, network, token0Address)

  let poolKey = newPoolKey(token0Address, token1Address, feeTier)
  beforeEach(async function () {
    this.timeout(200000)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    console.log('Adding fee tier')
    await invariant.addFeeTier(account, feeTier)

    console.log('Creating pool')
    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    console.log('Approving assets 0')
    psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    console.log('Approving assets 1')
    psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  })

  it('get tickmap', async function () {
    this.timeout(200000)
    const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
    console.log('Creating position')
    await invariant.createPosition(account, poolKey, ticks[2], ticks[3], 10n, pool.sqrtPrice, 0n)

    console.log('Queryin tickmap')
    try {
      const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
      assert.deepEqual(tickmap[3465], 9223372036854775809n)

      for (const [chunkIndex, value] of tickmap.entries()) {
        if (chunkIndex === 3465) {
          assert.deepEqual(
            value,
            0b1000000000000000000000000000000000000000000000000000000000000001n
          )
        } else {
          assert.deepEqual(value, 0n)
        }
      }
    } catch (e) {
      console.log(e)
    }
  })
  //   it('get tickmap edge ticks initialized', async () => {
  //     const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
  //     await invariant.createPosition(account, poolKey, ticks[0], ticks[1], 10n, pool.sqrtPrice, 0n)
  //     await invariant.createPosition(account, poolKey, ticks[4], ticks[5], 10n, pool.sqrtPrice, 0n)

  //     const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
  //     assert.deepEqual(tickmap[0], 0b11n)
  //     assert.deepEqual(
  //       tickmap[integerSafeCast(getMaxChunk(feeTier.tickSpacing))],
  //       0b11000000000000000000000000000000000000000000000000000n
  //     )
  //   })
  //   it('get tickmap more chunks above', async function () {
  //     this.timeout(2000000)

  //     const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

  //     for (let i = 6n; i < 52500n; i += 64n) {
  //       await invariant.createPosition(account, poolKey, i, i + 1n, 10n, pool.sqrtPrice, 0n)
  //     }

  //     const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)

  //     const initializedChunks = 52500n / 64n
  //     for (let i = 0n; i < initializedChunks; i++) {
  //       const current = 3466n + i
  //       assert.deepEqual(tickmap[integerSafeCast(current)], 0b11n)
  //     }
  //   })
  //   it('get tickmap more chunks below', async function () {
  //     this.timeout(2000000)

  //     const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

  //     // 51328
  //     for (let i = -52544n; i < 6n; i += 64n) {
  //       await invariant.createPosition(account, poolKey, i, i + 1n, 10n, pool.sqrtPrice, 0n)
  //     }

  //     const tickmap = await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
  //     const initializedChunks = 52544n / 64n
  //     for (let i = 0n; i < initializedChunks; i++) {
  //       const current = 2644n + i
  //       assert.deepEqual(
  //         tickmap[integerSafeCast(current)],
  //         0b110000000000000000000000000000000000000000000000000000000000n
  //       )
  //     }
  //   })
  //   it('get tickmap max chunks returned', async function () {
  //     this.timeout(2000000)

  //     const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

  //     for (let i = 0n; i < 104832n; i += 64n) {
  //       await invariant.createPosition(account, poolKey, i, i + 1n, 10n, pool.sqrtPrice, 0n)
  //     }

  //     await invariant.getTickmap(account, poolKey, pool.currentTickIndex)
  //   })
  //   it('get tickmap max chunks + 1 returned', async function () {
  //     this.timeout(2000000)

  //     const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

  //     for (let i = 0n; i < 104896n; i += 64n) {
  //       await invariant.createPosition(account, poolKey, i, i + 1n, 10n, pool.sqrtPrice, 0n)
  //     }

  //     assertThrowsAsync(invariant.getTickmap(account, poolKey, pool.currentTickIndex))
  //   })
})
