import dotenv from 'dotenv'
import {
  BTC_ADDRESS,
  ETH_ADDRESS,
  FEE_TIERS,
  filterTickmap,
  filterTicks,
  initPolkadotApi,
  Invariant,
  INVARIANT_ADDRESS,
  Keyring,
  MAX_SQRT_PRICE,
  MIN_SQRT_PRICE,
  Network,
  newPoolKey,
  PoolKey,
  PSP22,
  simulateInvariantSwap,
  SOL_ADDRESS,
  USDC_ADDRESS,
  USDT_ADDRESS,
  WAZERO_ADDRESS
} from '@invariant-labs/a0-sdk'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)
  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)
  console.log(`Trader: ${account.address}, Mnemonic: ${mnemonic}`)

  const invariant = await Invariant.load(api, network, INVARIANT_ADDRESS[network], {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  const poolKeys: [PoolKey, string][] = [
    [newPoolKey(WAZERO_ADDRESS[network], BTC_ADDRESS[network], FEE_TIERS[1]), 'WAZERO-BTC 1'],
    [newPoolKey(WAZERO_ADDRESS[network], ETH_ADDRESS[network], FEE_TIERS[1]), 'WAZERO-ETH 1'],
    [newPoolKey(WAZERO_ADDRESS[network], USDC_ADDRESS[network], FEE_TIERS[1]), 'WAZERO-USDC 1'],
    [newPoolKey(WAZERO_ADDRESS[network], USDT_ADDRESS[network], FEE_TIERS[1]), 'WAZERO-USDT 1'],
    [newPoolKey(WAZERO_ADDRESS[network], SOL_ADDRESS[network], FEE_TIERS[1]), 'WAZERO-SOL 1'],
    [newPoolKey(BTC_ADDRESS[network], ETH_ADDRESS[network], FEE_TIERS[1]), 'BTC-ETH 1'],
    [newPoolKey(BTC_ADDRESS[network], USDC_ADDRESS[network], FEE_TIERS[1]), 'BTC-USDC 1'],
    [newPoolKey(BTC_ADDRESS[network], USDT_ADDRESS[network], FEE_TIERS[1]), 'BTC-USDT 1'],
    [newPoolKey(BTC_ADDRESS[network], SOL_ADDRESS[network], FEE_TIERS[1]), 'BTC-SOL 1'],
    [newPoolKey(ETH_ADDRESS[network], USDC_ADDRESS[network], FEE_TIERS[1]), 'ETH-USDC 1'],
    [newPoolKey(ETH_ADDRESS[network], USDT_ADDRESS[network], FEE_TIERS[1]), 'ETH-USDT 1'],
    [newPoolKey(ETH_ADDRESS[network], SOL_ADDRESS[network], FEE_TIERS[1]), 'ETH-SOL 1'],
    [newPoolKey(USDC_ADDRESS[network], USDT_ADDRESS[network], FEE_TIERS[1]), 'USDC-USDT 1'],
    [newPoolKey(USDC_ADDRESS[network], SOL_ADDRESS[network], FEE_TIERS[1]), 'USDC-SOL 1'],
    [newPoolKey(USDT_ADDRESS[network], SOL_ADDRESS[network], FEE_TIERS[1]), 'USDT-SOL 1']
  ]

  const psp22 = await PSP22.load(api, network, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  let counter = 0
  while (true) {
    const [poolKey, name] = poolKeys[Math.floor(Math.random() * poolKeys.length)]

    console.log('pool: ', name)
    const pool = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)

    const xToY = Math.random() > 0.5
    const byAmountIn = Math.random() > 0.5

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
      byAmountIn,
      xToY ? MIN_SQRT_PRICE : MAX_SQRT_PRICE
    )
    console.log('Swap: ', ++counter)
    console.log('Simulation: ', simulation)
    console.log('xToY: ', xToY)
    console.log('byAmountIn: ', byAmountIn)
    const multiplier = Math.random() * 1.25
    const amount = (simulation.amountIn * BigInt(Math.trunc(multiplier * 100000))) / 100000n
    console.log('amountMultiplier: ', multiplier)
    console.log('amount: ', amount)
    const tokenAddress = xToY ? poolKey.tokenX : poolKey.tokenY
    if (!(counter % 1023)) {
      await api.disconnect()
      await delay(1000)
      await api.connect()
      await delay(1000)
    }
    try {
      await psp22.mint(account, amount, tokenAddress)
      await psp22.approve(account, invariant.contract.address.toString(), amount, tokenAddress)
      console.log(
        'Minted and approved:',
        await psp22.allowance(account.address, invariant.contract.address.toString(), tokenAddress)
      )
      const tx = await invariant.swap(
        account,
        poolKey,
        xToY,
        amount,
        byAmountIn,
        xToY ? MIN_SQRT_PRICE : MAX_SQRT_PRICE
      )
      console.log('success [', name, ']: ', tx)
    } catch (err: any) {
      console.log(`error: ${err.toString()}`)
      continue
    }
  }
}

const delay = (delayMs: number) => {
  return new Promise(resolve => setTimeout(resolve, delayMs))
}

main()
