import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { FeeTier } from '../src/schema'
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

  it('should change protocol fee', async () => {
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

  it('should add fee tier', async () => {
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

    const feeTier = new FeeTier(600, 5)
    const anotherFeeTier = new FeeTier(300, 10)

    await invariant.addFeeTier(account, feeTier)
    let feeTierExists = await invariant.feeTierExist(account, feeTier)
    let anotherFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(feeTierExists, true)
    assert.deepEqual(anotherFeeTierExists, false)
    assert.deepEqual(feeTiers, [feeTier])

    await invariant.addFeeTier(account, anotherFeeTier)
    feeTierExists = await invariant.feeTierExist(account, feeTier)
    anotherFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(feeTierExists, true)
    assert.deepEqual(anotherFeeTierExists, true)
    assert.deepEqual(feeTiers, [feeTier, anotherFeeTier])
  })

  it('should remove fee tier', async () => {
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

    const feeTier = new FeeTier(600, 5)
    const anotherFeeTier = new FeeTier(300, 10)

    await invariant.addFeeTier(account, feeTier)
    await invariant.addFeeTier(account, anotherFeeTier)

    await invariant.removeFeeTier(account, anotherFeeTier)

    let feeTierExists = await invariant.feeTierExist(account, feeTier)
    let anotherFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(feeTierExists, true)
    assert.deepEqual(anotherFeeTierExists, false)
    assert.deepEqual(feeTiers, [feeTier])

    await invariant.removeFeeTier(account, feeTier)

    feeTierExists = await invariant.feeTierExist(account, feeTier)
    anotherFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(feeTierExists, false)
    assert.deepEqual(anotherFeeTierExists, false)
    assert.deepEqual(feeTiers, [])
  })
})
