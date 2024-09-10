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
  const account = keyring.addFromMnemonic(mnemonic)
  console.log(`Deployer: ${account.address}, Mnemonic: ${mnemonic}`)

  const invariant = await Invariant.deploy(api, network, account, toPercentage(1n, 2n), {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  console.log(`Invariant: ${invariant.contract.address.toString()}`)

  const codeHash = await getCodeHash(api, invariant.contract.address.toString())

  const testnetInvariant = await Invariant.load(api, network, INVARIANT_ADDRESS[network], {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  await testnetInvariant.setCode(account, codeHash)

  process.exit(0)
}

main()
