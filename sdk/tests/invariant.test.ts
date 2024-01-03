import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { getDeploymentData, initPolkadotApi } from '../src/utils'

describe('invariant', function () {
  const init = async (): Promise<{ api: ApiPromise; account: IKeyringPair }> => {
    const api = await initPolkadotApi(Network.Local)

    const keyring = new Keyring({ type: 'sr25519' })
    const account = await keyring.addFromUri('//Alice')

    return { api, account }
  }

  it('deploys', async () => {
    const { api, account } = await init()

    const invariantData = await getDeploymentData('invariant')
    const invariant = new Invariant(api, Network.Local)

    const initFee = { v: 10 }
    const invariantDeploy = await invariant.deploy(
      account,
      invariantData.abi,
      invariantData.wasm,
      initFee
    )
    await invariant.load(invariantDeploy.address, invariantData.abi)
  })

  it('changes protocol fee', async () => {
    const { api, account } = await init()

    const invariantData = await getDeploymentData('invariant')
    const invariant = new Invariant(api, Network.Local)

    const initFee = { v: 10 }
    const invariantDeploy = await invariant.deploy(
      account,
      invariantData.abi,
      invariantData.wasm,
      initFee
    )
    await invariant.load(invariantDeploy.address, invariantData.abi)

    const newFeeStruct = {
      v: 100n
    }

    await invariant.changeProtocolFee(account, newFeeStruct)
    const newFee = await invariant.getProtocolFee(account)

    assert.deepEqual(newFee, { v: 100 })
  })
  // //TODO: needs PR with FeeTiers
  // it('create pool', async () => {
  //   const { api, account } = await init()

  //   const invariantData = await getDeploymentData('invariant')
  //   const invariant = new Invariant(api, Network.Local)

  //   const initFee = { v: 10 }
  //   const invariantDeploy = await invariant.deploy(
  //     account,
  //     invariantData.abi,
  //     invariantData.wasm,
  //     initFee
  //   )
  //   await invariant.load(invariantDeploy.address, invariantData.abi)

  //   const token0: string = '5H79vf7qQKdpefChp4sGh8j4BNq8JoL5x8nez8RsEebPJu9D'
  //   const token1: string = '5DxazQgoKEPMLqyUBRpqgAV7JnGv3w6i4EACTU8RDJxPHisH'
  //   const fee: Percentage = { v: 100n }
  //   const feeTier: FeeTier = newFeeTier(fee, 1)
  //   const initSqrtPrice: SqrtPrice = { v: 1000000000000000000n }
  //   const initTick = 1n

  //   const createPoolResult = await invariant.createPool(
  //     account,
  //     token0,
  //     token1,
  //     feeTier,
  //     initSqrtPrice,
  //     initTick
  //   )
  //   await sleep(1000)

  //   console.log(createPoolResult)

  //   const result = await invariant.getPool(account, token0, token1, feeTier)
  //   console.log(result)

  //   const pools = await invariant.getPools(account)
  //   console.log(pools)
  // })
})
