import { Keyring } from '@polkadot/api'
import dotenv from 'dotenv'
import { Network } from './network.js'
import {
  deployInvariant,
  deployPSP22,
  getEnvAccount,
  getEnvTestAccount,
  initPolkadotApi,
  printBalance
} from './utils.js'
dotenv.config()

import { getBalance, transferBalance } from '@scio-labs/use-inkathon'
import {
  FeeTier,
  Liquidity,
  PoolKey,
  SqrtPrice,
  getDecimalScales,
  getDeltaY,
  newFeeTier,
  newPoolKey
} from 'math/math.js'

const main = async () => {
  {
    const sqrtPriceA: SqrtPrice = {
      v: 234878324943782000000000000n
    }
    const sqrtPriceB: SqrtPrice = { v: 87854456421658000000000000n }
    const liquidity: Liquidity = { v: 983983249092n }
    const deltaYUp = getDeltaY(sqrtPriceA, sqrtPriceB, liquidity, true)
    const deltaYDown = getDeltaY(sqrtPriceA, sqrtPriceB, liquidity, false)
    console.log(deltaYUp)
    console.log(deltaYDown)
  }
  {
    const scales = getDecimalScales()
    console.log(scales)
  }
  {
    const feeTier: FeeTier = newFeeTier({ v: 10n }, 55)
    console.log(feeTier)
    const poolKey: PoolKey = newPoolKey(
      '5H79vf7qQKdpefChp4sGh8j4BNq8JoL5x8nez8RsEebPJu9D',
      '5DxazQgoKEPMLqyUBRpqgAV7JnGv3w6i4EACTU8RDJxPHisH',
      feeTier
    )
    console.log(poolKey)
  }

  const network = Network.getFromEnv()
  console.log(`using ${network}`)

  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const account = await getEnvAccount(keyring)
  const testAccount = await getEnvTestAccount(keyring)

  await printBalance(api, account)
  await printBalance(api, testAccount)

  // deploy invariant

  const initFee = { v: 10n }
  const invariant = await deployInvariant(api, account, initFee, Network.Local)

  // deploy token
  const token = await deployPSP22(api, account, 1000n, 'Coin', 'COIN', 12n)

  // // deploy wrapped azero
  // const wazeroData = await getDeploymentData('wrapped_azero')
  // const wazero = await WrappedAZERO.create(api, account, network)

  // if (process.env.WAZERO_ADDRESS && network !== Network.Local) {
  //   await wazero.load(process.env.WAZERO_ADDRESS, wazeroData.abi)
  //   console.log('loaded wazero')
  // } else {
  //   const wazeroDeploy = await wazero.deploy(account, wazeroData.abi, wazeroData.wasm)
  //   await wazero.load(wazeroDeploy.address, wazeroData.abi)
  //   console.log('deployed and loaded wazero')
  // }

  // change protocol fee
  const initialFee = await invariant.getProtocolFee(account)
  const newFeeStruct = {
    v: 18446744073709551615n
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

  // // wrap and unwrap azero
  // console.log('balance before deposit: ', await wazero.balanceOf(account, account.address))
  // await wazero.deposit(account, 1000000000000)
  // console.log('balance after deposit: ', await wazero.balanceOf(account, account.address))
  // await wazero.withdraw(account, 1000000000000)
  // console.log('balance after withdraw: ', await wazero.balanceOf(account, account.address))

  process.exit(0)
}

main()
