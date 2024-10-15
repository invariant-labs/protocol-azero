import dotenv from 'dotenv'
import {
  BTC_ADDRESS,
  ETH_ADDRESS,
  filterTickmap,
  filterTicks,
  initPolkadotApi,
  Invariant,
  INVARIANT_ADDRESS,
  Keyring,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  Network,
  PoolKey,
  PSP22,
  signAndSendTx,
  simulateInvariantSwap,
  SOL_ADDRESS,
  toPercentage,
  USDC_ADDRESS,
  USDT_ADDRESS
} from '@invariant-labs/a0-sdk'

dotenv.config()

const NETWORK = Network.Testnet
const FAUCET_TOKENS = new Map<string, string>([
  [BTC_ADDRESS[NETWORK], 'BTC'],
  [ETH_ADDRESS[NETWORK], 'ETH'],
  [USDC_ADDRESS[NETWORK], 'USDC'],
  [USDT_ADDRESS[NETWORK], 'USDT'],
  [SOL_ADDRESS[NETWORK], 'SOL']
])

const main = async () => {
  const api = await initPolkadotApi(NETWORK)
  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.USER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)
  console.log(`Trader: ${account.address}, Mnemonic: ${mnemonic}`)

  const invariant = await Invariant.load(api, NETWORK, INVARIANT_ADDRESS[NETWORK], {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  const psp22 = await PSP22.load(api, NETWORK, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  // IKeyRingPair is not re-exported in Invariant SDK
  const mintTokenUnderThreshold = async (tokenAddress: string, threshold: bigint) => {
    const balance = await psp22.balanceOf(account.address, tokenAddress)
    if (balance < threshold) {
      const mintTx = psp22.mintTx(threshold, tokenAddress)
      return [mintTx]
    }
    return []
  }

  const performSwap = async (poolKey: PoolKey, xToY: boolean) => {
    const byAmountIn = Math.random() > 0.5

    const pool = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)

    const tickmap = filterTickmap(
      await invariant.getFullTickmap(poolKey),
      poolKey.feeTier.tickSpacing,
      pool.currentTickIndex,
      xToY
    )
    const ticks = filterTicks(
      await invariant.getAllLiquidityTicks(poolKey, tickmap),
      pool.currentTickIndex,
      xToY
    )
    const simulation = simulateInvariantSwap(
      tickmap,
      poolKey.feeTier,
      pool,
      ticks,
      xToY,
      1n << (128n - 1n),
      true,
      xToY ? MIN_SQRT_PRICE : MAX_SQRT_PRICE
    )

    const multiplier = Math.random() * 1.25
    const amount =
      ((byAmountIn ? simulation.amountIn : simulation.amountOut) *
        BigInt(Math.trunc(multiplier * 100000))) /
      100000n

    return invariant.swapTx(
      poolKey,
      xToY,
      amount,
      byAmountIn,
      xToY ? MIN_SQRT_PRICE : MAX_SQRT_PRICE
    )
  }

  let pools: { poolKey: PoolKey; id: string }[] = []
  const poolKeys = await invariant.getAllPoolKeys()
  for (const poolKey of poolKeys) {
    if (isSupportedToken(poolKey.tokenX) && isSupportedToken(poolKey.tokenY)) {
      pools.push({
        poolKey,
        id:
          FAUCET_TOKENS.get(poolKey.tokenX) +
          '-' +
          FAUCET_TOKENS.get(poolKey.tokenY) +
          ' ' +
          Number(poolKey.feeTier.fee) / Number(toPercentage(1, 2)) +
          '%' +
          ' ' +
          poolKey.feeTier.tickSpacing
      })
    }
  }

  for (const [tokenAddress] of FAUCET_TOKENS) {
    await psp22.approve(
      account,
      invariant.contract.address.toString(),
      1n << (128n - 1n),
      tokenAddress
    )
  }

  let attemptCounter = 0
  let successCounter = 0

  while (true) {
    const { poolKey, id } = pools[Math.floor(Math.random() * pools.length)]
    const xToY = Math.random() > 0.5

    try {
      attemptCounter += 1
      if (!(attemptCounter % 1023)) {
        await api.disconnect()
        await delay(1000)
        await api.connect()
        await delay(1000)
      }

      const { sqrtPrice: sqrtPriceBefore } = await invariant.getPool(
        poolKey.tokenX,
        poolKey.tokenY,
        poolKey.feeTier
      )

      let txBatchArray = await mintTokenUnderThreshold(
        xToY ? poolKey.tokenX : poolKey.tokenY,
        1n << 99n
      )
      txBatchArray.push(await performSwap(poolKey, xToY))
      const txBatch = api.tx.utility.batch(txBatchArray)

      await signAndSendTx(txBatch, account, true, true)

      console.log('---------------')
      const { sqrtPrice } = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)

      if (sqrtPrice !== sqrtPriceBefore) {
        ++successCounter
        console.log('success [', id, ']')
      } else {
        console.log('failure [', id, ']')
      }

      console.log('Attempt counter: ', attemptCounter)
      console.log('Success percentage: ', (successCounter / attemptCounter) * 100, '%')
    } catch (e) {
      console.error(e)
    }
  }
}

const isSupportedToken = (address: string): boolean => {
  return FAUCET_TOKENS.has(address)
}

const delay = (delayMs: number) => {
  return new Promise(resolve => setTimeout(resolve, delayMs))
}

main()
