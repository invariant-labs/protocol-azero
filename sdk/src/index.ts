import dotenv from 'dotenv'
import { Liquidity, SqrtPrice, getDeltaY } from '../math/pkg/math.js'
// import { Invariant } from './invariant.js'
// import { Network } from './network.js'
// import { getDeploymentData, getEnvAccount, initPolkadotApi, printBalance } from './utils.js'

dotenv.config()

const main = async () => {
  {
    let sqrtPriceA: SqrtPrice = {
      v: BigInt(234878324943782000000000000)
    }
    let sqrtPriceB: SqrtPrice = { v: BigInt(87854456421658000000000000) }
    let liquidity: Liquidity = { v: BigInt(983983249092) }
    let delta_y_up = getDeltaY(sqrtPriceA, sqrtPriceB, liquidity, true)
    let delta_y_down = getDeltaY(sqrtPriceA, sqrtPriceB, liquidity, false)
    console.log(delta_y_up)
    console.log(delta_y_down)
  }
  // const network = Network.getFromEnv()
  // console.log(`Using ${network}`)
  // const api = await initPolkadotApi(network)
  // const keyring = new Keyring({ type: 'sr25519' })
  // const account = await getEnvAccount(keyring)
  // await printBalance(api, account)

  // const { abi, wasm } = await getDeploymentData()
  // const invariant = new Invariant(api, account, network)

  // const initFee = { v: 10 }
  // const deployContract = await invariant.deploy(abi, wasm, initFee)
  // await invariant.load(deployContract.address, abi)

  // const initialFee = await invariant.getProtocolFee()
  // console.log(initialFee)

  // const newFeeStruct = {
  //   v: 18446744073709551615n
  // }

  // console.log(`Changing protocol fee to: ${newFeeStruct.v}`)

  // const txHash = await invariant.changeProtocolFee(newFeeStruct)

  // console.log('Received txHash  = ', txHash)

  // const newFee = await invariant.getProtocolFee()
  // console.log(newFee)

  // console.log('Passed.')
  process.exit(0)
}

main()
