import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { newFeeTier } from 'math/math.js'
import { Network } from '../src/network'
import { deployInvariant, initPolkadotApi } from '../src/utils'

describe('invariant', async () => {
  const api = await initPolkadotApi(Network.Local)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = await keyring.addFromUri('//Alice')

  let invariant = await deployInvariant(api, account, { v: 10000000000n })

  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n })
  })

  it('should change protocol fee', async () => {
    const newFeeStruct = { v: 20000000000n }

    await invariant.changeProtocolFee(account, newFeeStruct)
    const newFee = await invariant.getProtocolFee(account)

    assert.equal(newFee.v, newFeeStruct.v)
  })

  it('should add fee tier', async () => {
    const feeTier = newFeeTier({ v: 10000000000n }, 5)
    const anotherFeeTier = newFeeTier({ v: 20000000000n }, 10)

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
    const feeTier = newFeeTier({ v: 10000000000n }, 5)
    const anotherFeeTier = newFeeTier({ v: 20000000000n }, 10)

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
