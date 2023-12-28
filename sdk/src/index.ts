import { Keyring } from '@polkadot/api'
import { getBalance, transferBalance } from '@scio-labs/use-inkathon'
import dotenv from 'dotenv'
import { Invariant } from './invariant.js'
import { Network } from './network.js'
import { PSP22 } from './psp22.js'
import {
  getDeploymentData,
  getEnvAccount,
  getEnvTestAccount,
  initPolkadotApi,
  printBalance
} from './utils.js'
import { WrappedAZERO } from './wrapped_azero.js'
dotenv.config()

const main = async () => {
  const network = Network.getFromEnv()
  console.log(`using ${network}`)

  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = await getEnvAccount(keyring)
  const testAccount = await getEnvTestAccount(keyring)

  await printBalance(api, account)
  await printBalance(api, testAccount)

  // deploy invariant
  const invariantData = await getDeploymentData('contract')
  const invariant = new Invariant(api, network)

  const initFee = { v: 10 }
  const invariantDeploy = await invariant.deploy(
    account,
    invariantData.abi,
    invariantData.wasm,
    initFee
  )
  await invariant.load(invariantDeploy.address, invariantData.abi)

  // deploy token
  const tokenData = await getDeploymentData('psp22')
  const token = new PSP22(api, network)

  const name = api.createType('Option<String>', 'Coin')
  const symbol = api.createType('Option<String>', 'COIN')

  const tokenDeploy = await token.deploy(
    account,
    tokenData.abi,
    tokenData.wasm,
    1000,
    name,
    symbol,
    0
  )
  await token.load(tokenDeploy.address, tokenData.abi)

  // deploy wrapped azero
  const wazeroData = await getDeploymentData('wrapped_azero')
  const wazero = new WrappedAZERO(api, network)

  const wazeroDeploy = await wazero.deploy(account, wazeroData.abi, wazeroData.wasm)
  await wazero.load(wazeroDeploy.address, wazeroData.abi)

  // change protocol fee
  const initialFee = await invariant.getProtocolFee(account)
  const newFeeStruct = {
    v: 100
  }
  await invariant.changeProtocolFee(account, newFeeStruct)
  const newFee = await invariant.getProtocolFee(account)
  console.log('old fee: ', initialFee, ', new fee: ', newFee)

  // perform token operations
  await token.mint(account, 500)

  console.log(
    'token name: ',
    await token.tokenName(account),
    ', token symbol: ',
    await token.tokenSymbol(account),
    ', token decimals: ',
    await token.tokenDecimals(account)
  )

  const data = api.createType('Vec<u8>', [])
  await token.transfer(account, account.address, 250, data)

  await transferBalance(api, account, testAccount.address, 1000000000000)
  console.log('account balance: ', (await getBalance(api, account.address)).balanceFormatted)
  console.log(
    'test account balance: ',
    (await getBalance(api, testAccount.address)).balanceFormatted
  )

  // wrap and unwrap azero
  console.log('balance before deposit: ', await wazero.balanceOf(account, account.address))
  await wazero.deposit(account, 1000000000000)
  console.log('balance after deposit: ', await wazero.balanceOf(account, account.address))
  await wazero.withdraw(account, 1000000000000)
  console.log('balance after withdraw: ', await wazero.balanceOf(account, account.address))

  process.exit(0)
}

main()
