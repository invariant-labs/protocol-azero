import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { toPercentage } from '../wasm/pkg/invariant_a0_wasm'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const feeTier = newFeeTier(10000000000n, 1n)

let invariant = await Invariant.deploy(api, Network.Local, account, toPercentage(1n, 2n))
let token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

const psp22 = await PSP22.load(api, Network.Local, token0Address)

describe('limits', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)

    poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  })

  it('should be able to store more than 256kb of data in storage', async function () {
    this.timeout(35000)

    for (let i = 1n; i <= 1619n; i++) {
      const result = await invariant.createPosition(
        account,
        poolKey,
        -1n,
        1n,
        1000n,
        1000000000000000000000000n,
        0n
      )
      assert.equal(result.events.length, 1)
    }
  })
})
