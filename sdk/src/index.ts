import { Keyring } from '@polkadot/api'
import dotenv from 'dotenv'
import { Invariant } from './invariant.js'
import { Network } from './network.js'
import { getDeploymentData, getEnvAccount, initPolkadotApi, printBalance } from './utils.js'
dotenv.config()

const main = async () => {
  const network = Network.getFromEnv()
  console.log(`Using ${network}`)
  const api = await initPolkadotApi(network)
  const keyring = new Keyring({ type: 'sr25519' })
  const account = await getEnvAccount(keyring)
  await printBalance(api, account)

  const { abi, wasm } = await getDeploymentData()
  const invariant = new Invariant(api, account, network)

  const initFee = { v: 10 }
  const deployContract = await invariant.deploy(abi, wasm, initFee)
  await invariant.load(deployContract.address, abi)

  const initialFee = await invariant.getProtocolFee()
  console.log(initialFee)

  const newFeeStruct = {
    v: 100
  }

  console.log(`Changing protocol fee to: ${newFeeStruct.v}`)

  const txHash = await invariant.changeProtocolFee(newFeeStruct)

  console.log('Received txHash  = ', txHash)

  const newFee = await invariant.getProtocolFee()
  console.log(newFee)

  console.log('Passed.')
  process.exit(0)
}

main()
