import {
  FEE_TIERS,
  Invariant,
  Keyring,
  Network,
  PSP22,
  PoolKey,
  TESTNET_WAZERO_ADDRESS,
  WrappedAZERO,
  calculateTick,
  initPolkadotApi,
  newPoolKey,
  priceToSqrtPrice,
  toPercentage
} from '@invariant-labs/a0-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)
  console.log(`Deployer: ${account.address}, Mnemonic: ${mnemonic}`)

  const invariant = await Invariant.deploy(api, network, account, toPercentage(1n, 2n), {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  console.log(`Invariant: ${invariant.contract.address.toString()}`)

  for (const feeTier of FEE_TIERS) {
    await invariant.addFeeTier(account, feeTier)
  }
  console.log('Successfully added fee tiers')

  const BTCAddress = await PSP22.deploy(api, account, 0n, 'Bitcoin', 'BTC', 8n)
  const ETHAddress = await PSP22.deploy(api, account, 0n, 'Ether', 'ETH', 12n)
  const USDCAddress = await PSP22.deploy(api, account, 0n, 'USDC', 'USDC', 6n)
  const USDTAddress = await PSP22.deploy(api, account, 0n, 'Tether USD', 'USDT', 6n)
  const SOLAddress = await PSP22.deploy(api, account, 0n, 'Solana', 'SOL', 9n)
  const decimals = {
    [BTCAddress]: 8n,
    [ETHAddress]: 12n,
    [USDCAddress]: 6n,
    [USDTAddress]: 6n,
    [SOLAddress]: 9n,
    [TESTNET_WAZERO_ADDRESS]: 12n
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
    [TESTNET_WAZERO_ADDRESS]: data.find((coin: any) => coin.id === 'aleph-zero').current_price
  }
  console.log(
    `BTC: ${prices[BTCAddress]}, ETH: ${prices[ETHAddress]}, USDC: ${prices[USDCAddress]}, USDT: ${prices[USDTAddress]}, SOL: ${prices[SOLAddress]}, AZERO: ${prices[TESTNET_WAZERO_ADDRESS]}`
  )

  const poolKeys: [PoolKey, bigint][] = [
    [newPoolKey(TESTNET_WAZERO_ADDRESS, BTCAddress, FEE_TIERS[1]), 10804609546189987720n],
    [newPoolKey(TESTNET_WAZERO_ADDRESS, ETHAddress, FEE_TIERS[1]), 4711830510277394610468n],
    [newPoolKey(TESTNET_WAZERO_ADDRESS, USDCAddress, FEE_TIERS[1]), 272063075569508447756n],
    [newPoolKey(TESTNET_WAZERO_ADDRESS, USDTAddress, FEE_TIERS[1]), 272063075569508447756n],
    [newPoolKey(TESTNET_WAZERO_ADDRESS, SOLAddress, FEE_TIERS[1]), 37143700245489847211n],
    [newPoolKey(BTCAddress, ETHAddress, FEE_TIERS[1]), 130559235944405760n],
    [newPoolKey(BTCAddress, USDCAddress, FEE_TIERS[1]), 7865049221247086n],
    [newPoolKey(BTCAddress, USDTAddress, FEE_TIERS[1]), 7865049221247086n],
    [newPoolKey(BTCAddress, SOLAddress, FEE_TIERS[1]), 977937074251981n],
    [newPoolKey(ETHAddress, USDCAddress, FEE_TIERS[1]), 3454809855596621497n],
    [newPoolKey(ETHAddress, USDTAddress, FEE_TIERS[1]), 3454809855596621497n],
    [newPoolKey(ETHAddress, SOLAddress, FEE_TIERS[1]), 423131631710393596n],
    [newPoolKey(USDCAddress, USDTAddress, FEE_TIERS[1]), 9999818389598293n],
    [newPoolKey(USDCAddress, SOLAddress, FEE_TIERS[1]), 24911294718392400n],
    [newPoolKey(USDTAddress, SOLAddress, FEE_TIERS[1]), 24911294718392400n]
  ]
  for (const [poolKey] of poolKeys) {
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

  const psp22 = await PSP22.load(api, network, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
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
  const wazero = await WrappedAZERO.load(api, network, TESTNET_WAZERO_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  const wazeroBalance = await wazero.balanceOf(account.address)
  await wazero.withdraw(account, wazeroBalance)
  await wazero.deposit(account, 50000n * 10n ** 12n)
  await psp22.approve(
    account,
    invariant.contract.address.toString(),
    2n ** 96n - 1n,
    TESTNET_WAZERO_ADDRESS
  )
  const BTCBefore = await psp22.balanceOf(account.address, BTCAddress)
  const ETHBefore = await psp22.balanceOf(account.address, ETHAddress)
  const USDCBefore = await psp22.balanceOf(account.address, USDCAddress)
  const USDTBefore = await psp22.balanceOf(account.address, USDTAddress)
  const SOLBefore = await psp22.balanceOf(account.address, SOLAddress)
  const WAZEROBefore = await psp22.balanceOf(account.address, TESTNET_WAZERO_ADDRESS)
  for (const [poolKey, amount] of poolKeys) {
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
      await invariant.createPosition(
        account,
        poolKey,
        lowerTick,
        upperTick,
        amount,
        poolSqrtPrice,
        0n
      )
    } catch (e) {
      console.log('Create position error', poolKey, e)
    }
  }
  const BTCAfter = await psp22.balanceOf(account.address, BTCAddress)
  const ETHAfter = await psp22.balanceOf(account.address, ETHAddress)
  const USDCAfter = await psp22.balanceOf(account.address, USDCAddress)
  const USDTAfter = await psp22.balanceOf(account.address, USDTAddress)
  const SOLAfter = await psp22.balanceOf(account.address, SOLAddress)
  const WAZEROAfter = await psp22.balanceOf(account.address, TESTNET_WAZERO_ADDRESS)
  console.log(
    `BTC: ${BTCBefore - BTCAfter}, ETH: ${ETHBefore - ETHAfter}, USDC: ${
      USDCBefore - USDCAfter
    }, USDT: ${USDTBefore - USDTAfter}, SOL: ${SOLBefore - SOLAfter}, AZERO: ${
      WAZEROBefore - WAZEROAfter
    }`
  )
  console.log('Successfully created positions')

  process.exit(0)
}

main()
