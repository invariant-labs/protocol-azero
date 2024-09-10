import {
  Invariant,
  Keyring,
  Network,
  BTC_ADDRESS,
  ETH_ADDRESS,
  INVARIANT_ADDRESS,
  initPolkadotApi,
  newFeeTier,
  newPoolKey,
  toPercentage,
  toSqrtPrice
} from '@invariant-labs/a0-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)

  const FEE_TIER = newFeeTier(toPercentage(1n, 4n), 1n)
  const TOKEN_0_ADDRESS = ETH_ADDRESS[network]
  const TOKEN_1_ADDRESS = BTC_ADDRESS[network]
  const POOL_KEY = newPoolKey(TOKEN_0_ADDRESS, TOKEN_1_ADDRESS, FEE_TIER)

  const invariant = await Invariant.load(api, network, INVARIANT_ADDRESS[network], {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  await invariant.createPool(account, POOL_KEY, 1000000000000000000000000n)

  process.exit(0)
}

main()
