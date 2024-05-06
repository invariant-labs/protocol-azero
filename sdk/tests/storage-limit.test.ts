import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { toPercentage } from 'invariant-a0-wasm/invariant_a0_wasm.js'
import { DEFAULT_PROOF_SIZE, MAX_REF_TIME } from '../src/consts'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils'

describe('storage limit', async () => {
  it('storage limit test', async function () {
    this.timeout(1000000000000)

    const api = await initPolkadotApi(Network.Testnet)

    const keyring = new Keyring({ type: 'sr25519' })
    const account = await keyring.addFromUri('//Alice')
    const feeTier = newFeeTier(10000000000n, 1n)

    const invariant = await Invariant.deploy(api, Network.Testnet, account, toPercentage(1n, 2n), {
      storageDepositLimit: null,
      refTime: MAX_REF_TIME,
      proofSize: DEFAULT_PROOF_SIZE
    })
    const token0Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    const token1Address = await PSP22.deploy(api, account, 1000000000000n, 'Coin', 'COIN', 0n)
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    const psp22 = await PSP22.load(api, Network.Testnet, token0Address, {
      storageDepositLimit: null,
      refTime: MAX_REF_TIME,
      proofSize: DEFAULT_PROOF_SIZE
    })

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(account, poolKey, 1000000000000000000000000n)

    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
    await psp22.setContractAddress(token1Address)
    await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)

    // 1619 * 162 B = 262278 B > 256 KB
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
