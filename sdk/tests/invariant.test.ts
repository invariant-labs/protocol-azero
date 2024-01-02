import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { assert } from 'chai'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { FeeTier, Type } from '../src/schema'
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

    const initFee = { v: 10000000000n }
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

    const initFee = { v: 10000000000n }
    const invariantDeploy = await invariant.deploy(
      account,
      invariantData.abi,
      invariantData.wasm,
      initFee
    )
    await invariant.load(invariantDeploy.address, invariantData.abi)

    const newFeeStruct = new Type(20000000000n)

    await invariant.changeProtocolFee(account, newFeeStruct)
    const newFee = await invariant.getProtocolFee(account)

    assert.equal(newFee.v, newFeeStruct.v)
  })

  it('should add fee tier', async () => {
    const { api, account } = await init()

    const invariantData = await getDeploymentData('invariant')
    const invariant = new Invariant(api, Network.Local)

    const initFee = { v: 10000000000n }
    const invariantDeploy = await invariant.deploy(
      account,
      invariantData.abi,
      invariantData.wasm,
      initFee
    )
    await invariant.load(invariantDeploy.address, invariantData.abi)

    const feeTier = new FeeTier(10000000000n, 5n)
    const anotherFeeTier = new FeeTier(20000000000n, 10n)

    await invariant.addFeeTier(account, feeTier)
    let addedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    const notAddedFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(addedFeeTierExists, true)
    assert.deepEqual(notAddedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.addFeeTier(account, anotherFeeTier)
    const addedBeforeFeeTierExists = await invariant.feeTierExist(account, feeTier)
    addedFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(addedBeforeFeeTierExists, true)
    assert.deepEqual(addedFeeTierExists, true)
    assert.deepEqual(feeTiers.length, 2)
  })

  it('should remove fee tier', async () => {
    const { api, account } = await init()

    const invariantData = await getDeploymentData('invariant')
    const invariant = new Invariant(api, Network.Local)

    const initFee = { v: 10000000000n }
    const invariantDeploy = await invariant.deploy(
      account,
      invariantData.abi,
      invariantData.wasm,
      initFee
    )
    await invariant.load(invariantDeploy.address, invariantData.abi)

    const feeTier = new FeeTier(10000000000n, 5n)
    const anotherFeeTier = new FeeTier(20000000000n, 10n)

    await invariant.addFeeTier(account, feeTier)
    await invariant.addFeeTier(account, anotherFeeTier)

    await invariant.removeFeeTier(account, anotherFeeTier)
    const notRemovedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    let removedFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    let feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(notRemovedFeeTierExists, true)
    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 1)

    await invariant.removeFeeTier(account, feeTier)
    removedFeeTierExists = await invariant.feeTierExist(account, feeTier)
    const removedBeforeFeeTierExists = await invariant.feeTierExist(account, anotherFeeTier)
    feeTiers = await invariant.getFeeTiers(account)

    assert.deepEqual(removedFeeTierExists, false)
    assert.deepEqual(removedBeforeFeeTierExists, false)
    assert.deepEqual(feeTiers.length, 0)
  })
})
