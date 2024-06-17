import {
  Invariant,
  Keyring,
  Network,
  PSP22,
  PoolKey,
  TESTNET_WAZERO_ADDRESS,
  WrappedAZERO,
  calculateTick,
  initPolkadotApi,
  newFeeTier,
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

  const hundredthOfPercentage = toPercentage(1n, 4n)
  const feeTiers = [1n, 2n, 5n, 10n, 30n, 100n].map(tickCount =>
    newFeeTier(tickCount * hundredthOfPercentage, tickCount)
  )
  for (const feeTier of feeTiers) {
    await invariant.addFeeTier(account, feeTier)
  }
  console.log('Successfully added fee tiers')

  const BTCAddress = await PSP22.deploy(api, account, 0n, 'Bitcoin', 'BTC', 8n)
  const ETHAddress = await PSP22.deploy(api, account, 0n, 'Ether', 'ETH', 12n)
  const USDCAddress = await PSP22.deploy(api, account, 0n, 'USDC', 'USDC', 6n)
  const decimals = {
    [BTCAddress]: 8n,
    [ETHAddress]: 12n,
    [USDCAddress]: 6n,
    [TESTNET_WAZERO_ADDRESS]: 12n
  }
  console.log(`BTC: ${BTCAddress}, ETH: ${ETHAddress}, USDC: ${USDCAddress}`)

  const response = await fetch(
    'https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids=bitcoin,ethereum,aleph-zero'
  )
  const data = await response.json()
  const prices = {
    [BTCAddress]: data.find((coin: any) => coin.id === 'bitcoin').current_price,
    [ETHAddress]: data.find((coin: any) => coin.id === 'ethereum').current_price,
    [USDCAddress]: 1,
    [TESTNET_WAZERO_ADDRESS]: data.find((coin: any) => coin.id === 'aleph-zero').current_price
  }
  console.log(
    `BTC: ${prices[BTCAddress]}, ETH: ${prices[ETHAddress]}, USDC: ${prices[USDCAddress]}, AZERO: ${prices[TESTNET_WAZERO_ADDRESS]}`
  )

  const poolKeys: [PoolKey, bigint][] = [
    [newPoolKey(TESTNET_WAZERO_ADDRESS, BTCAddress, feeTiers[1]), 10804609546189987720n],
    [newPoolKey(TESTNET_WAZERO_ADDRESS, ETHAddress, feeTiers[1]), 4711830510277394610468n],
    [newPoolKey(TESTNET_WAZERO_ADDRESS, USDCAddress, feeTiers[1]), 272063075569508447756n],
    [newPoolKey(BTCAddress, ETHAddress, feeTiers[1]), 130559235944405760n],
    [newPoolKey(BTCAddress, USDCAddress, feeTiers[1]), 7865049221247086n],
    [newPoolKey(ETHAddress, USDCAddress, feeTiers[1]), 3454809855596621497n]
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
  await psp22.mint(account, 2n ** 128n - 1n, BTCAddress)
  await psp22.mint(account, 2n ** 128n - 1n, ETHAddress)
  await psp22.mint(account, 2n ** 128n - 1n, USDCAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 128n - 1n, BTCAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 128n - 1n, ETHAddress)
  await psp22.approve(account, invariant.contract.address.toString(), 2n ** 128n - 1n, USDCAddress)
  const wazero = await WrappedAZERO.load(api, network, TESTNET_WAZERO_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  const wazeroBalance = await wazero.balanceOf(account.address)
  await wazero.withdraw(account, wazeroBalance)
  await wazero.deposit(account, 40000n * 10n ** 12n)
  await psp22.approve(
    account,
    invariant.contract.address.toString(),
    2n ** 128n - 1n,
    TESTNET_WAZERO_ADDRESS
  )
  const BTCBefore = await psp22.balanceOf(account.address, BTCAddress)
  const ETHBefore = await psp22.balanceOf(account.address, ETHAddress)
  const USDCBefore = await psp22.balanceOf(account.address, USDCAddress)
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
      const lowerTick = calculateTick(lowerSqrtPrice, feeTiers[1].tickSpacing)
      const upperTick = calculateTick(upperSqrtPrice, feeTiers[1].tickSpacing)
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
  const WAZEROAfter = await psp22.balanceOf(account.address, TESTNET_WAZERO_ADDRESS)
  console.log(
    `BTC: ${BTCBefore - BTCAfter}, ETH: ${ETHBefore - ETHAfter}, USDC: ${
      USDCBefore - USDCAfter
    }, AZERO: ${WAZEROBefore - WAZEROAfter}`
  )
  console.log('Successfully created positions')

  process.exit(0)
}

main()
