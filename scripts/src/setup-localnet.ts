import {
  FEE_TIERS,
  Invariant,
  Keyring,
  Network,
  PSP22,
  PoolKey,
  WrappedAZERO,
  calculateTick,
  getLiquidityByX,
  initPolkadotApi,
  newPoolKey,
  priceToSqrtPrice,
  toPercentage
} from '@invariant-labs/a0-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Local
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = keyring.addFromUri('//Alice')
  console.log(`Deployer: ${account.address}`)

  const invariant = await Invariant.deploy(api, network, account, toPercentage(1n, 2n))
  const WAZERO = await WrappedAZERO.deploy(api, network, account)
  console.log(`Invariant: ${invariant.contract.address.toString()}`)

  for (const feeTier of FEE_TIERS) {
    await invariant.addFeeTier(account, feeTier)
  }
  console.log('Successfully added fee tiers')

  const WAZEROAddress = WAZERO.contract.address.toString()
  const BTCAddress = await PSP22.deploy(api, account, 0n, 'Bitcoin', 'BTC', 8n)
  const ETHAddress = await PSP22.deploy(api, account, 0n, 'Ether', 'ETH', 18n)
  const USDCAddress = await PSP22.deploy(api, account, 0n, 'USDC', 'USDC', 6n)
  const USDTAddress = await PSP22.deploy(api, account, 0n, 'Tether USD', 'USDT', 6n)
  const SOLAddress = await PSP22.deploy(api, account, 0n, 'Solana', 'SOL', 9n)
  const decimals = {
    [BTCAddress]: 8n,
    [ETHAddress]: 12n,
    [USDCAddress]: 6n,
    [USDTAddress]: 6n,
    [SOLAddress]: 9n,
    [WAZEROAddress]: 12n
  }
  console.log(
    `BTC: ${BTCAddress}, ETH: ${ETHAddress}, USDC: ${USDCAddress}, USDT: ${USDTAddress}, SOL: ${SOLAddress}`
  )

  const response = await fetch(
    'https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids=bitcoin,ethereum,aleph-zero,solana'
  )
  const data = await response.json()
  const prices = {
    [BTCAddress]: data.find((coin: any) => coin.id === 'bitcoin').current_price,
    [ETHAddress]: data.find((coin: any) => coin.id === 'ethereum').current_price,
    [USDCAddress]: 1,
    [USDTAddress]: 1,
    [SOLAddress]: data.find((coin: any) => coin.id === 'solana').current_price,
    [WAZEROAddress]: data.find((coin: any) => coin.id === 'aleph-zero').current_price
  }
  console.log(
    `BTC: ${prices[BTCAddress]}, ETH: ${prices[ETHAddress]}, USDC: ${prices[USDCAddress]}, USDT: ${prices[USDTAddress]}, SOL: ${prices[SOLAddress]}, AZERO: ${prices[WAZEROAddress]}`
  )

  const poolKeys: PoolKey[] = [
    newPoolKey(WAZEROAddress, BTCAddress, FEE_TIERS[1]),
    newPoolKey(WAZEROAddress, ETHAddress, FEE_TIERS[1]),
    newPoolKey(WAZEROAddress, USDCAddress, FEE_TIERS[1]),
    newPoolKey(WAZEROAddress, USDTAddress, FEE_TIERS[1]),
    newPoolKey(WAZEROAddress, SOLAddress, FEE_TIERS[1]),
    newPoolKey(BTCAddress, ETHAddress, FEE_TIERS[1]),
    newPoolKey(BTCAddress, USDCAddress, FEE_TIERS[1]),
    newPoolKey(BTCAddress, USDTAddress, FEE_TIERS[1]),
    newPoolKey(BTCAddress, SOLAddress, FEE_TIERS[1]),
    newPoolKey(ETHAddress, USDCAddress, FEE_TIERS[1]),
    newPoolKey(ETHAddress, USDTAddress, FEE_TIERS[1]),
    newPoolKey(ETHAddress, SOLAddress, FEE_TIERS[1]),
    newPoolKey(USDCAddress, USDTAddress, FEE_TIERS[1]),
    newPoolKey(USDCAddress, SOLAddress, FEE_TIERS[1]),
    newPoolKey(USDTAddress, SOLAddress, FEE_TIERS[1])
  ]
  for (const poolKey of poolKeys) {
    const price =
      (1 / (prices[poolKey.tokenY] / prices[poolKey.tokenX])) *
      10 ** (Number(decimals[poolKey.tokenY]) - Number(decimals[poolKey.tokenX])) *
      10 ** 24
    try {
      const poolSqrtPrice = priceToSqrtPrice(BigInt(Math.round(price)))
      await invariant.createPool(account, poolKey, poolSqrtPrice)
    } catch (e) {
      console.log('Create pool error', poolKey, e)
    }
  }
  console.log('Successfully added pools')

  const psp22 = await PSP22.load(api, network)
  await psp22.mint(account, 2n ** 96n - 1n, BTCAddress)
  await psp22.mint(account, 2n ** 96n - 1n, ETHAddress)
  await psp22.mint(account, 2n ** 96n - 1n, USDCAddress)
  await psp22.mint(account, 2n ** 96n - 1n, USDTAddress)
  await psp22.mint(account, 2n ** 96n - 1n, SOLAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 96n - 1n, BTCAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 96n - 1n, ETHAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 96n - 1n, USDCAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 96n - 1n, USDTAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 96n - 1n, SOLAddress)
  const wazero = await WrappedAZERO.load(api, network, WAZEROAddress)
  const wazeroBalance = await wazero.balanceOf(account.address)
  await wazero.withdraw(account, wazeroBalance)
  await wazero.deposit(account, 100000n * 10n ** 12n)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 96n - 1n, WAZEROAddress)
  for (const poolKey of poolKeys) {
    const price =
      (1 / (prices[poolKey.tokenY] / prices[poolKey.tokenX])) *
      10 ** (Number(decimals[poolKey.tokenY]) - Number(decimals[poolKey.tokenX])) *
      10 ** 24
    const lowerSqrtPrice = priceToSqrtPrice(BigInt(Math.round(price * 0.95)))
    const upperSqrtPrice = priceToSqrtPrice(BigInt(Math.round(price * 1.05)))
    const poolSqrtPrice = priceToSqrtPrice(BigInt(Math.round(price)))
    try {
      const lowerTick = calculateTick(lowerSqrtPrice, FEE_TIERS[1].tickSpacing)
      const upperTick = calculateTick(upperSqrtPrice, FEE_TIERS[1].tickSpacing)
      const tokenXAmount = BigInt(
        Math.round((5000 / prices[poolKey.tokenX]) * 10 ** Number(decimals[poolKey.tokenX]))
      )
      const { l: liquidity } = getLiquidityByX(
        tokenXAmount,
        lowerTick,
        upperTick,
        poolSqrtPrice,
        true
      )
      await invariant.createPosition(
        account,
        poolKey,
        lowerTick,
        upperTick,
        liquidity,
        poolSqrtPrice,
        0n
      )
    } catch (e) {
      console.log('Create position error', poolKey, e)
    }
  }
  console.log('Successfully created positions')

  process.exit(0)
}

main()
