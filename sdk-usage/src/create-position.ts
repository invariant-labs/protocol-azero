import {
  Invariant,
  Keyring,
  Network,
  PSP22,
  TESTNET_ETH_ADDRESS,
  TESTNET_INVARIANT_ADDRESS,
  TETSNET_USDC_ADDRESS,
  initPolkadotApi,
  newFeeTier,
  newPoolKey,
  toPercentage
} from '@invariant-labs/a0-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const receiver = process.env.RECEIVER_ADDRESS ?? ''
  const account = keyring.addFromMnemonic(mnemonic)

  const FEE_TIER = newFeeTier(toPercentage(1n, 4n), 1n)
  const TOKEN_0_ADDRESS = TETSNET_USDC_ADDRESS
  const TOKEN_1_ADDRESS = TESTNET_ETH_ADDRESS
  const POOL_KEY = newPoolKey(TOKEN_0_ADDRESS, TOKEN_1_ADDRESS, FEE_TIER)

  const invariant = await Invariant.load(api, network, TESTNET_INVARIANT_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  const psp22 = await PSP22.load(api, network, TOKEN_0_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)

  await psp22.approve(account, TESTNET_INVARIANT_ADDRESS, 1000000n)
  console.log(
    `Token 0 balance: ${await psp22.balanceOf(
      account.address,
      account.address
    )}, Token 0 allowance: ${await psp22.allowance(
      account.address,
      account.address,
      TESTNET_INVARIANT_ADDRESS
    )}`
  )

  psp22.setContractAddress(TOKEN_1_ADDRESS)

  await psp22.approve(account, TESTNET_INVARIANT_ADDRESS, 1000000n)
  console.log(
    `Token 1 balance: ${await psp22.balanceOf(
      account.address,
      account.address
    )}, Token 1 allowance: ${await psp22.allowance(
      account.address,
      account.address,
      TESTNET_INVARIANT_ADDRESS
    )}`
  )

  await invariant.createPosition(
    account,
    POOL_KEY,
    -10n,
    10n,
    1000000n,
    1000000000000000000000000n,
    0n
  )

  if (receiver) {
    await invariant.transferPosition(account, 0n, receiver)
  }

  process.exit(0)
}

main()
