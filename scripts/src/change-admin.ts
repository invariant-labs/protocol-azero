import {
  Invariant,
  Keyring,
  Network,
  INVARIANT_ADDRESS,
  getCodeHash,
  initPolkadotApi,
  toPercentage
} from '@invariant-labs/a0-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const newMnemonic = process.env.NEW_DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)
  const newAccount = keyring.addFromMnemonic(newMnemonic)
  console.log(
    `Deployer: ${account.address}, Mnemonic: ${mnemonic}, New Deployer: ${newAccount.address}, New Mnemonic: ${newMnemonic}`
  )

  const invariant = await Invariant.load(api, network, INVARIANT_ADDRESS[network], {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  await invariant.changeAdmin(account, newAccount.address)

  process.exit(0)
}

main()
