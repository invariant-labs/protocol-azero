import { Keyring } from '@polkadot/api'
import { Invariant } from '../src/invariant.js'
import { Network } from '../src/network.js'
import { PSP22 } from '../src/psp22.js'
import { ContractOptions } from '../src/schema.js'
import { getMinTick, getMaxTick, initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils.js'
import { describe, it } from 'mocha'
import { CHUNK_SIZE, MAX_TICKMAP_QUERY_SIZE } from '../src/consts.js'
import { assert } from 'chai'
import { tickIndexToPosition } from '@invariant-labs/a0-sdk-wasm/invariant_a0_wasm.js'

describe('testnet-tickmap-limitations', async () => {
  it('Validate tickmap query size on max tick spread', async function () {
    this.timeout(2000000)
    const network = Network.Testnet

    const api = await initPolkadotApi(network)
    const keyring = new Keyring({ type: 'sr25519' })
    const deployer = keyring.addFromUri('//Alice')

    const deployOptions: ContractOptions = {
      storageDepositLimit: null,
      refTime: 259058343000,
      proofSize: 1160117
    }

    const invariant = await Invariant.deploy(api, network, deployer, 10000000000n, deployOptions)
    const token0Address = await PSP22.deploy(api, deployer, 1000000000n)
    const token1Address = await PSP22.deploy(api, deployer, 1000000000n)
    const psp22 = await PSP22.load(api, network, deployOptions)

    const feeTier = newFeeTier(0n, 1n)
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(deployer, feeTier)
    const spotSqrtPrice = 1000000000000000000000000n
    await invariant.createPool(deployer, poolKey, spotSqrtPrice)

    const mintAmount = 1n << 110n
    await psp22.mint(deployer, mintAmount, token0Address)
    await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount, token0Address)
    await psp22.mint(deployer, mintAmount, token1Address)
    await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount, token1Address)

    const liquidityDelta = 100n
    const slippageTolerance = 0n

    const tickCount = MAX_TICKMAP_QUERY_SIZE

    // add position with (0, spread) indexes if this is not 0
    assert.equal(tickCount % 2n, 0n, 'tick count not divisible by 2')
    
    const spread = (getMaxTick(1n) / MAX_TICKMAP_QUERY_SIZE) * 2n
    const lookupSize = 64n
    const limitTick = (tickCount / 2n) * spread

    assert(spread < CHUNK_SIZE * lookupSize, 'spread too small')

    {
      const [chunk] = tickIndexToPosition(getMaxTick(1n), 1n)
      const [targetChunk] = tickIndexToPosition(limitTick, 1n)

      assert.equal(
        BigInt(chunk) / lookupSize,
        BigInt(targetChunk) / lookupSize,
        'Max chunk mismatch'
      )
    }
    {
      const lookupSize = 64n
      const [chunk] = tickIndexToPosition(getMinTick(1n), 1n)
      const [targetChunk] = tickIndexToPosition(-limitTick, 1n)

      assert.equal(
        BigInt(chunk) / lookupSize,
        BigInt(targetChunk) / lookupSize,
        'Min chunk mismatch'
      )
    }

    for (let i = 1n; i <= tickCount / 2n; i++) {
      console.log(-i * spread, i * spread)
      await invariant.createPosition(
        deployer,
        poolKey,
        -i * spread,
        i * spread,
        liquidityDelta,
        spotSqrtPrice,
        slippageTolerance
      )
    }

    const xToY = await invariant.getRawTickmap(poolKey, getMinTick(1n), getMaxTick(1n), true)
    const yToX = await invariant.getRawTickmap(poolKey, getMinTick(1n), getMaxTick(1n), false)
    assert.equal(xToY.length, Number(MAX_TICKMAP_QUERY_SIZE))
    assert.equal(yToX.length, Number(MAX_TICKMAP_QUERY_SIZE))
  })
})
