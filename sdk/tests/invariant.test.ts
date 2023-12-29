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
})
