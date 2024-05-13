import {
  Keyring,
  Network,
  Invariant,
  initPolkadotApi,
  newFeeTier,
  toPercentage,
  FeeTier,
  DEFAULT_PROOF_SIZE,
  newPoolKey,
  TESTNET_ETH_ADDRESS,
  TETSNET_USDC_ADDRESS,  
} from '@invariant-labs/a0-sdk'
import { ContractOptions } from '@invariant-labs/a0-sdk/target/schema'
import dotenv from 'dotenv'


dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)

  const deployOptions: ContractOptions = {
    storageDepositLimit: null,
    refTime: 259058343000,
    proofSize: DEFAULT_PROOF_SIZE
  }

  const INVARIANT = await Invariant.deploy(
    api,
    network,
    account,
    toPercentage(1n, 2n),
    deployOptions
  )

  const hundredthOfPercentage = toPercentage(1n, 4n)
  const generateFee = (tickCount: bigint): FeeTier => {
    return newFeeTier(tickCount * hundredthOfPercentage, tickCount)
  }

  console.log(`Invariant: ${INVARIANT.contract.address.toString()}`)
  console.log(`Deployer: ${account.address}, Mnemonic: ${mnemonic}`)

  const feeTiers = [
    generateFee(1n),
    generateFee(2n),
    generateFee(5n),
    generateFee(10n),
    generateFee(30n),
    generateFee(100n)
  ]

  for (const feeTier of feeTiers) {
    await INVARIANT.addFeeTier(account, feeTier).catch(err => {
      console.error(err), process.exit(1)
    })
    console.log(`Fee tier added: ${feeTier.fee}, ${feeTier.tickSpacing}`)
  }

  await INVARIANT.createPool(
    account,
    newPoolKey(TETSNET_USDC_ADDRESS, TESTNET_ETH_ADDRESS, feeTiers[0]),
    1000000000000000000000000n
  ).catch(err => {
    console.error(err), process.exit(1)
  })

  console.log(`Pool added ${TETSNET_USDC_ADDRESS}, ${TESTNET_ETH_ADDRESS}`)
  process.exit(0)
}

main()
