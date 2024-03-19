import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from '../src/invariant.js'
import { Network } from '../src/network.js'
import { PSP22 } from '../src/psp22.js'
import { ContractOptions } from '../src/schema.js'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils.js'

describe('get-tickmap-testnet', async () => {
  it('Query tickmap from contract', async function () {
    this.timeout(2000000)
    const network = Network.Testnet
    const ticks = [-221818n, -221817n, -58n, 5n, 221817n, 221818n]
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
    const psp22 = await PSP22.load(api, network, token0Address, deployOptions)

    const feeTier = newFeeTier(6000000000n, 1n)
    const poolKey = newPoolKey(token0Address, token1Address, feeTier)

    await invariant.addFeeTier(deployer, feeTier)

    await invariant.createPool(deployer, poolKey, 1000000000000000000000000n)

    const mintAmount = 1n << 110n
    await psp22.setContractAddress(token0Address)
    await psp22.mint(deployer, mintAmount)
    await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount)
    await psp22.setContractAddress(token1Address)
    await psp22.mint(deployer, mintAmount)
    await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount)

    // const liquidityDelta = 10n ** 6n
    // const spotSqrtPrice = 1000000000000000000000000n
    // const slippageTolerance = 0n

    const pool = await invariant.getPool(deployer, token0Address, token1Address, feeTier)

    // await invariant.createPosition(
    //   deployer,
    //   poolKey,
    //   ticks[2],
    //   ticks[3],
    //   liquidityDelta,
    //   spotSqrtPrice,
    //   slippageTolerance
    // )

    const tickmap = await invariant.getTickmap(deployer, poolKey, pool.currentTickIndex)
    assert.deepEqual(tickmap[3465], 9223372036854775809n)

    for (const [chunkIndex, value] of tickmap.entries()) {
      if (chunkIndex === 3465) {
        assert.deepEqual(value, 0b1000000000000000000000000000000000000000000000000000000000000001n)
      } else {
        assert.deepEqual(value, 0n)
      }
    }
  })
})
