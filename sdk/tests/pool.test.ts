import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { assert } from 'chai'
import { SqrtPrice, newFeeTier } from 'math/math.js'
import { Network } from '../src/network'
import { deployInvariant, initPolkadotApi, sleep } from '../src/utils'

describe('invariant', async () => {
  const api = await initPolkadotApi(Network.Local)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = await keyring.addFromUri('//Alice')

  let invariant = await deployInvariant(api, account, { v: 10000000000n })
  // let token_0 = await tokenToString.deplo

  const deployToken = async (): Promise<{
    api: ApiPromise
    account: IKeyringPair
    testAccount: IKeyringPair
  }> => {
    const api = await initPolkadotApi(Network.Local)

    const keyring = new Keyring({ type: 'sr25519' })
    const account = await keyring.addFromUri('//Alice')
    const testAccount = await keyring.addFromUri('//Bob')

    return { api, account, testAccount }
  }

  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n })
  })

  it('create pool', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 5)
    await invariant.addFeeTier(account, feeTier)
    let addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    assert.deepEqual(addedFeeTierExists, true)

    const token0: string = '5H79vf7qQKdpefChp4sGh8j4BNq8JoL5x8nez8RsEebPJu9D'
    const token1: string = '5DxazQgoKEPMLqyUBRpqgAV7JnGv3w6i4EACTU8RDJxPHisH'
    const initSqrtPrice: SqrtPrice = { v: 1000000000000000000n }
    const initTick = 1n

    const createPoolResult = await invariant.createPool(
      account,
      token0,
      token1,
      feeTier,
      initSqrtPrice,
      initTick
    )

    await sleep(1000)

    console.log(createPoolResult)

    const result = await invariant.getPool(account, token0, token1, feeTier)
    console.log(result)

    const pools = await invariant.getPools(account)
    console.log(pools)
  })
})
