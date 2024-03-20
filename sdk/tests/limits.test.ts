import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'
import { toPercentage } from '../src/wasm/pkg/invariant_a0_wasm.js'

const api = await initPolkadotApi(Network.Testnet)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const feeTier = newFeeTier(10000000000n, 1n)

let invariant = await Invariant.deploy(api, Network.Testnet, account, toPercentage(1n, 2n))
let token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
let poolKey = newPoolKey(token0Address, token1Address, feeTier)

const psp22 = await PSP22.load(api, Network.Testnet, token0Address)

describe('limits', async () => {
  beforeEach(async function () {
    this.timeout(60000)

    invariant = await Invariant.deploy(api, Network.Testnet, account, 10000000000n)
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

  it('storage limit test', async function () {
    this.timeout(1000000000000)

    for (let i = 1; i <= 1619; i++) {
      console.log(i)
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
