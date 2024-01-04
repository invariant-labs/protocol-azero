import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { InvariantError, newFeeTier, newPoolKey } from 'math/math.js'
import { Network } from '../src/network'
import { assertError, deployInvariant, deployPSP22, initPolkadotApi } from '../src/utils'

describe('invariant', async () => {
  const api = await initPolkadotApi(Network.Local)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = await keyring.addFromUri('//Alice')

  let invariant = await deployInvariant(api, account, { v: 10000000000n })
  let token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n)
  let token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n)

  beforeEach(async () => {
    invariant = await deployInvariant(api, account, { v: 10000000000n })
    token0 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n)
    token1 = await deployPSP22(api, account, 1000000000n, 'Coin', 'COIN', 0n)
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

  it('should get tick and check if it is initialized', async () => {
    if (!token0.contract?.address || !token1.contract?.address || !invariant.contract?.address) {
      throw new Error()
    }

    const feeTier = newFeeTier({ v: 10000000000n }, 1)

    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    await token0.approve(account, invariant.contract.address.toString(), 1000000000n)
    await token1.approve(account, invariant.contract.address.toString(), 1000000000n)

    const poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    await invariant.createPosition(
      account,
      poolKey,
      -10n,
      10n,
      { v: 1000000n },
      { v: 0n },
      { v: 100000000000000000000000000n }
    )

    const lowerTick = await invariant.getTick(account, poolKey, -10n)
    assert.deepEqual(lowerTick, {
      index: -10n,
      sign: true,
      liquidityChange: { v: 1000000n },
      liquidityGross: { v: 1000000n },
      sqrtPrice: { v: 999500149965000000000000n },
      feeGrowthOutsideX: { v: 0n },
      feeGrowthOutsideY: { v: 0n },
      secondsOutside: lowerTick.secondsOutside
    })
    await assertError(invariant.getTick(account, poolKey, 0n), InvariantError.TickNotFound)
    const upperTick = await invariant.getTick(account, poolKey, 10n)
    assert.deepEqual(upperTick, {
      index: 10n,
      sign: false,
      liquidityChange: { v: 1000000n },
      liquidityGross: { v: 1000000n },
      sqrtPrice: { v: 1000500100010000000000000n },
      feeGrowthOutsideX: { v: 0n },
      feeGrowthOutsideY: { v: 0n },
      secondsOutside: upperTick.secondsOutside
    })

    const isLowerTickInitialized = await invariant.isTickInitialized(account, poolKey, -10n)
    assert.deepEqual(isLowerTickInitialized, true)
    const isInitTickInitialized = await invariant.isTickInitialized(account, poolKey, 0n)
    assert.deepEqual(isInitTickInitialized, false)
    const isUpperTickInitialized = await invariant.isTickInitialized(account, poolKey, 10n)
    assert.deepEqual(isUpperTickInitialized, true)
  })
  it('create position', async () => {
    if (!token0.contract?.address || !token1.contract?.address || !invariant.contract?.address) {
      throw new Error()
    }

    const feeTier = newFeeTier({ v: 10000000000n }, 1)
    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    await token0.approve(account, invariant.contract.address.toString(), 1000000000n)
    await token1.approve(account, invariant.contract.address.toString(), 1000000000n)

    const poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    const lowerTickIndex = -10n
    const upperTickIndex = 10n

    await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      { v: 1000000n },
      { v: 0n },
      { v: 100000000000000000000000000n }
    )
  })
  it('remove position', async () => {
    if (!token0.contract?.address || !token1.contract?.address || !invariant.contract?.address) {
      throw new Error()
    }

    const feeTier = newFeeTier({ v: 10000000000n }, 1)
    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    await token0.approve(account, invariant.contract.address.toString(), 1000000000n)
    await token1.approve(account, invariant.contract.address.toString(), 1000000000n)

    const poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    const lowerTickIndex = -10n
    const upperTickIndex = 10n

    {
      await invariant.createPosition(
        account,
        poolKey,
        lowerTickIndex,
        upperTickIndex,
        { v: 1000000n },
        { v: 0n },
        { v: 100000000000000000000000000n }
      )
      const position = await invariant.getPosition(account, 0n)

      assert.deepEqual(position.liquidity, { v: 1000000n })
      assert.deepEqual(position.lowerTickIndex, lowerTickIndex)
      assert.deepEqual(position.upperTickIndex, upperTickIndex)
      assert.deepEqual(position.feeGrowthInsideX, { v: 0n })
      assert.deepEqual(position.feeGrowthInsideY, { v: 0n })
      assert.deepEqual(position.tokensOwedX, 0n)
      assert.deepEqual(position.tokensOwedY, 0n)
    }
    {
      await invariant.removePosition(account, 0n)
      // assertThrowsAsync(
      //   invariant.getPosition(account, 0n),
      //   InvariantError.PositionNotFound
      // )
      const positions = await invariant.getPositions(account)
      assert.deepEqual(positions.length, 0)
    }
    {
      // assertThrowsAsync(
      //   invariant.getTick(account, poolKey, lowerTickIndex),
      //   InvariantError.TickNotFound
      // )

      // assertThrowsAsync(
      //   invariant.getTick(account, poolKey, upperTickIndex),
      //   InvariantError.TickNotFound
      // )

      const isLowerTickInitialized = await invariant.isTickInitialized(
        account,
        poolKey,
        lowerTickIndex
      )
      assert.exists(!isLowerTickInitialized)

      const isUpperTickInitialized = await invariant.isTickInitialized(
        account,
        poolKey,
        upperTickIndex
      )

      assert.exists(!isUpperTickInitialized)
    }
  })
  it('transfer position', async () => {
    if (!token0.contract?.address || !token1.contract?.address || !invariant.contract?.address) {
      throw new Error()
    }

    const feeTier = newFeeTier({ v: 10000000000n }, 1)
    await invariant.addFeeTier(account, feeTier)

    await invariant.createPool(
      account,
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier,
      { v: 1000000000000000000000000n },
      0n
    )

    await token0.approve(account, invariant.contract.address.toString(), 1000000000n)
    await token1.approve(account, invariant.contract.address.toString(), 1000000000n)

    const poolKey = newPoolKey(
      token0.contract.address.toString(),
      token1.contract.address.toString(),
      feeTier
    )

    const lowerTickIndex = -10n
    const upperTickIndex = 10n

    await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      { v: 1000000n },
      { v: 0n },
      { v: 100000000000000000000000000n }
    )
    {
      // TODO add get position assertions
      const receiver = await keyring.addFromUri('//Bob')
      await invariant.transferPosition(account, 0n, receiver.address)
    }
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
