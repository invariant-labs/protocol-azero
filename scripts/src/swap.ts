import {
  Invariant,
  Keyring,
  Network,
  PSP22,
  PoolKey,
  SwapEvent,
  INVARIANT_ADDRESS,
  calculateFee,
  initPolkadotApi,
  positionToTick,
  simulateInvariantSwap
} from '@invariant-labs/a0-sdk'
import { CHUNK_SIZE } from '@invariant-labs/a0-sdk/target/consts.js'
import assert from 'assert'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)

  const POSITION_ID = 0n
  const SWAP_AMOUNT = 1000000n

  const invariant = await Invariant.load(api, network, INVARIANT_ADDRESS[network], {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  const positionBefore = await invariant.getPosition(account.address, POSITION_ID)
  const psp22 = await PSP22.load(api, network, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)

  await psp22.mint(account, SWAP_AMOUNT, positionBefore.poolKey.tokenX)
  await psp22.approve(
    account,
    INVARIANT_ADDRESS[network],
    SWAP_AMOUNT,
    positionBefore.poolKey.tokenX
  )

  await psp22.mint(account, SWAP_AMOUNT, positionBefore.poolKey.tokenY)
  await psp22.approve(
    account,
    INVARIANT_ADDRESS[network],
    SWAP_AMOUNT,
    positionBefore.poolKey.tokenY
  )
  const {
    tickmap: tickmapBeforeFirstSwap,
    ticks: ticksBeforeFirstSwap,
    pool: poolBeforeFirstSwap
  } = await getPoolState(invariant, positionBefore.poolKey)

  const firstSimualtion = simulateInvariantSwap(
    tickmapBeforeFirstSwap,
    positionBefore.poolKey.feeTier,
    poolBeforeFirstSwap,
    ticksBeforeFirstSwap,
    true,
    SWAP_AMOUNT,
    true,
    0n
  )
  const firstSwapResult = await invariant.swap(
    account,
    positionBefore.poolKey,
    true,
    SWAP_AMOUNT,
    true,
    0n
  )
  const firstSwapEvent = firstSwapResult.events[0] as SwapEvent
  assert(firstSimualtion.globalInsufficientLiquidity === false)
  assert(firstSimualtion.maxTicksCrossed === false)
  assert(firstSimualtion.stateOutdated === false)
  assert(firstSimualtion.amountIn == firstSwapEvent.amountIn)
  assert(firstSimualtion.amountOut == firstSwapEvent.amountOut)
  assert(firstSimualtion.crossedTicks.length === 0)
  assert(firstSimualtion.startSqrtPrice === firstSwapEvent.startSqrtPrice)
  assert(firstSimualtion.targetSqrtPrice === firstSwapEvent.targetSqrtPrice)

  const {
    tickmap: tickmapBeforeSecondSwap,
    ticks: ticksBeforeSecondSwap,
    pool: poolBeforeSecondSwap
  } = await getPoolState(invariant, positionBefore.poolKey)

  const secondSimulation = simulateInvariantSwap(
    tickmapBeforeSecondSwap,
    positionBefore.poolKey.feeTier,
    poolBeforeSecondSwap,
    ticksBeforeSecondSwap,
    false,
    SWAP_AMOUNT,
    true,
    2n ** 128n - 1n
  )
  const secondSwapResult = await invariant.swap(
    account,
    positionBefore.poolKey,
    false,
    SWAP_AMOUNT,
    true,
    2n ** 128n - 1n
  )
  const secondSwapEvent = secondSwapResult.events[0] as SwapEvent
  assert(secondSimulation.globalInsufficientLiquidity === false)
  assert(secondSimulation.maxTicksCrossed === false)
  assert(secondSimulation.stateOutdated === false)
  assert(secondSimulation.amountIn === secondSwapEvent.amountIn)
  assert(secondSimulation.amountOut === secondSwapEvent.amountOut)
  assert(secondSimulation.crossedTicks.length === 0)
  assert(secondSimulation.startSqrtPrice === secondSwapEvent.startSqrtPrice)
  assert(secondSimulation.targetSqrtPrice === secondSwapEvent.targetSqrtPrice)

  const pool = await invariant.getPool(
    positionBefore.poolKey.tokenX,
    positionBefore.poolKey.tokenY,
    positionBefore.poolKey.feeTier
  )
  console.log('Pool:', pool)
  const positionAfter = await invariant.getPosition(account.address, POSITION_ID)
  const lowerTick = await invariant.getTick(positionBefore.poolKey, positionBefore.lowerTickIndex)
  const upperTick = await invariant.getTick(positionBefore.poolKey, positionBefore.upperTickIndex)
  console.log('Fees:', calculateFee(pool, positionAfter, lowerTick, upperTick))

  process.exit(0)
}
const getPoolState = async (invariant: Invariant, poolKey: PoolKey) => {
  console.log(poolKey)
  const tickmap = await invariant.getFullTickmap(poolKey)
  const promises = []
  for (const [chunkIndex, chunk] of tickmap.bitmap.entries()) {
    for (let bit = 0n; bit < CHUNK_SIZE; bit++) {
      const checkedBit = chunk & (1n << bit)
      if (checkedBit) {
        const tickIndex = positionToTick(chunkIndex, bit, poolKey.feeTier.tickSpacing)
        promises.push(invariant.getTick(poolKey, tickIndex))
      }
    }
  }

  const ticks = await Promise.all(promises)

  const pool = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)
  return { tickmap, ticks, pool }
}
main()
