import { Keyring, Network, PSP22, initPolkadotApi } from '@invariant-labs/a0-sdk'

const main = async () => {
  const api = await initPolkadotApi(Network.Testnet)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = keyring.addFromUri('//Alice')

  const BTC_ADDRESS = await PSP22.deploy(api, account, 0n, 'Bitcoin', 'BTC', 8n)
  const ETH_ADDRESS = await PSP22.deploy(api, account, 0n, 'Ether', 'ETH', 18n)
  const USDC_ADDRESS = await PSP22.deploy(api, account, 0n, 'USDC', 'USDC', 6n)

  console.log(`Deployer: ${account.address}, Uri: //Alice`)
  console.log(`BTC: ${BTC_ADDRESS}, ETH: ${ETH_ADDRESS}, USDC: ${USDC_ADDRESS}`)

  process.exit(0)
}

main()
