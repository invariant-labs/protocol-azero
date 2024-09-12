import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { CrossTickEvent, getMinSqrtPrice, isTokenX } from '../src/index.js'
import { Invariant } from '../src/invariant.js'
import { Network } from '../src/network.js'
import { PSP22 } from '../src/psp22.js'
import { ContractOptions } from '../src/schema.js'
import { initPolkadotApi, newFeeTier, newPoolKey, simulateInvariantSwap } from '../src/utils.js'
import { describe, it } from 'mocha'

describe('testnet-crosses-limitations', async () => {
  it('Validate limitation number of crosses in single atomic swap', async function () {
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
    const token0Address = await PSP22.deploy(api, deployer, 1000000000n, 'Coin', 'COIN', 0n)
    const token1Address = await PSP22.deploy(api, deployer, 1000000000n, 'Coin', 'COIN', 0n)
    const psp22 = await PSP22.load(api, network, deployOptions)

    const feeTier = newFeeTier(6000000000n, 1n)
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(deployer, feeTier)
    const spotSqrtPrice = 1000000000000000000000000n
    await invariant.createPool(deployer, poolKey, spotSqrtPrice)

    const mintAmount = 1n << 110n
    await psp22.mint(deployer, mintAmount, token0Address)
    await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount, token0Address)
    await psp22.mint(deployer, mintAmount, token1Address)
    await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount, token1Address)

    const liquidityDelta = 1000000n * 10n ** 6n
    const slippageTolerance = 0n

    const tickCount = 160n
    const spread = 256n
    for (let i = 0n; i < tickCount; i++) {
      console.log(-i * spread - spread)
      await invariant.createPosition(
        deployer,
        poolKey,
        -i * spread - spread,
        -i * spread,
        liquidityDelta,
        spotSqrtPrice,
        slippageTolerance
      )
    }

    const swapper = keyring.addFromUri('//Bob')
    const swapAmount = 2177323n
    const tokenX = isTokenX(token0Address, token1Address) ? token0Address : token1Address

    await psp22.mint(swapper, swapAmount, tokenX)
    await psp22.approve(swapper, invariant.contract.address.toString(), swapAmount, tokenX)
    const poolBeforeSwap = await invariant.getPool(token0Address, token1Address, feeTier)
    const tickmap = await invariant.getFullTickmap(poolKey)
    const liquidityTicks = await invariant.getAllLiquidityTicks(poolKey, tickmap)

    const sim = simulateInvariantSwap(
      tickmap,
      feeTier,
      poolBeforeSwap,
      liquidityTicks,
      true,
      swapAmount,
      true,
      getMinSqrtPrice(poolKey.feeTier.tickSpacing)
    )

    const targetSqrtPrice = sim.targetSqrtPrice

    const tx = await invariant.swap(swapper, poolKey, true, swapAmount, true, targetSqrtPrice)
    assert.equal((tx.events[0] as CrossTickEvent).indexes.length, 90)
  })
})
