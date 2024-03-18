import { Keyring } from '@polkadot/api'
import { ContractOptions } from 'src/schema.js'
import { Invariant } from '../src/invariant.js'
import { Network } from '../src/network.js'
import { PSP22 } from '../src/psp22.js'
import { objectEquals } from '../src/testUtils.js'
import { initPolkadotApi, newFeeTier, newPoolKey } from '../src/utils.js'
import { CreatePositionEvent, Position } from '../src/wasm/pkg/invariant_a0_wasm.js'

describe.only('crosses', async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)
  const keyring = new Keyring({ type: 'sr25519' })
  const account = keyring.addFromUri('//Alice')

  const deployOptions: ContractOptions = {
    storageDepositLimit: null,
    refTime: 259058343000,
    proofSize: 160117
  }

  const invariant = await Invariant.deploy(api, network, account, 10000000000n, deployOptions)
  const token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
  const token1Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
  console.log('Tokens deployed!')
  const psp22 = await PSP22.load(api, network, token0Address, deployOptions)

  const lowerTickIndex = -20n
  const upperTickIndex = 10n

  const feeTier = newFeeTier(6000000000n, 10n)
  const poolKey = newPoolKey(token0Address, token1Address, feeTier)

  console.log('Adding fee Tier...')
  console.log(feeTier)
  // RefTimeLimit = 4288656576
  // ProofSize = 118551
  await invariant.addFeeTier(account, feeTier)
  console.log('Fee tier added')
  console.log('Creating pool...')
  // RefTimeLimit = 4957550834
  // ProofSize = 120331
  await invariant.createPool(account, poolKey, 1000000000000000000000000n)
  console.log('Pool created')

  await psp22.setContractAddress(token0Address)
  await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  await psp22.setContractAddress(token1Address)
  await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)

  // const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)
  console.log('Pool Queried')

  console.log('Creating position...')
  // RefTimeLimit = 9142042117
  // ProofSize = 160117
  const result = await invariant.createPosition(
    account,
    poolKey,
    lowerTickIndex,
    upperTickIndex,
    100n,
    1000000000000000000000000n,
    0n
  )
  console.log('Position opened!')
  const expectedCreatePositionEvent: CreatePositionEvent = {
    address: account.address.toString(),
    currentSqrtPrice: 1000000000000000000000000n,
    liquidity: 1000000000000n,
    lowerTick: -20n,
    pool: poolKey,
    upperTick: 10n,
    timestamp: 0n
  }

  objectEquals(result.events[0], expectedCreatePositionEvent, ['timestamp'])

  const position = await invariant.getPosition(account, account.address, 0n)
  const expectedPosition: Position = {
    poolKey: poolKey,
    liquidity: 1000000000000n,
    lowerTickIndex: lowerTickIndex,
    upperTickIndex: upperTickIndex,
    feeGrowthInsideX: 0n,
    feeGrowthInsideY: 0n,
    lastBlockNumber: 0n,
    tokensOwedX: 0n,
    tokensOwedY: 0n
  }
  objectEquals(position, expectedPosition, ['lastBlockNumber'])
})
