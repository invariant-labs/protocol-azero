import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { isTokenX } from '../src/index.js'
import { Invariant } from '../src/invariant.js'
import { Network } from '../src/network.js'
import { PSP22 } from '../src/psp22.js'
import { ContractOptions } from '../src/schema.js'
import { initPolkadotApi, integerSafeCast, newFeeTier, newPoolKey } from '../src/utils.js'

describe.only('crosses', async () => {
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
  console.log('Tokens deployed!')
  const psp22 = await PSP22.load(api, network, token0Address, deployOptions)

  const feeTier = newFeeTier(6000000000n, 10n)
  const poolKey = newPoolKey(token0Address, token1Address, feeTier)

  console.log('Adding fee Tier...')
  console.log(feeTier)
  // RefTimeLimit = 4288656576
  // ProofSize = 118551
  await invariant.addFeeTier(deployer, feeTier)
  console.log('Fee tier added')
  console.log('Creating pool...')
  // RefTimeLimit = 4957550834
  // ProofSize = 120331
  await invariant.createPool(deployer, poolKey, 1000000000000000000000000n)
  console.log('Pool created')

  const mintAmount = 1n << 110n
  await psp22.setContractAddress(token0Address)
  await psp22.mint(deployer, mintAmount)
  await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount)
  await psp22.setContractAddress(token1Address)
  await psp22.mint(deployer, mintAmount)
  await psp22.approve(deployer, invariant.contract.address.toString(), mintAmount)

  const pool = await invariant.getPool(deployer, token0Address, token1Address, feeTier)
  console.log('Pool Queried')
  console.log(pool)

  console.log('Creating position...')
  // RefTimeLimit = 9142042117
  // ProofSize = 160117
  const liquidityDelta = 10000000n * 10n ** 6n
  const spotSqrtPrice = 1000000000000000000000000n
  const slippageTolerance = 0n
  let positionOpenedCounter = 0
  for (let i = -2560n; i < 20; i += 10n) {
    await invariant.createPosition(
      deployer,
      poolKey,
      i,
      i + 10n,
      liquidityDelta,
      spotSqrtPrice,
      slippageTolerance
    )
    console.log('Position created: ', positionOpenedCounter++)
  }

  const position = await invariant.getPosition(deployer, deployer.address, 0n)
  console.log('position queried')
  console.log(position)

  const swapper = keyring.addFromUri('//Bob')
  // 1000000n - fails
  // 950000n - fails
  // 925000n - fails
  // 912500n - quote passes, swap fails - 174 quote ticks crossed
  // 910000n - quote passes, swap fails - 174 quote ticks crossed
  // 909500n - quote passes, swap fails - 174 quote ticks crossed
  // 909000n - pass - 173 crosses
  // 908000n - pass - 173 crosses
  // 906750n - pass - 173 crosses
  // 860000n - pass
  const swapAmount = 909500n
  const tokenX = isTokenX(token0Address, token1Address) ? token0Address : token1Address

  psp22.setContractAddress(tokenX)
  await psp22.mint(swapper, swapAmount)
  await psp22.approve(swapper, invariant.contract.address.toString(), swapAmount)

  const quote = await invariant.quote(swapper, poolKey, true, swapAmount, true)
  const targetSqrtPrice = quote.targetSqrtPrice

  const poolBeforeSwap = await invariant.getPool(deployer, token0Address, token1Address, feeTier)

  await invariant.swap(swapper, poolKey, true, swapAmount, true, targetSqrtPrice)

  const poolAfterSwap = await invariant.getPool(deployer, token0Address, token1Address, feeTier)

  const crossed = Math.abs(
    integerSafeCast((poolAfterSwap.currentTickIndex - poolBeforeSwap.currentTickIndex) / 10n)
  )
  assert.equal(crossed, 173)
})
