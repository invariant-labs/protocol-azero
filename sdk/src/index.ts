import { Keyring } from '@polkadot/api'
import dotenv from 'dotenv'
import { Network } from './network.js'
import {
  DEFAULT_PROOF_SIZE,
  DEFAULT_REF_TIME,
  getEnvAccount,
  initPolkadotApi,
  printBalance
} from './utils.js'
import { WrappedAZERO } from './wrapped_azero.js'
dotenv.config()

import { getBalance, transferBalance } from '@scio-labs/use-inkathon'
import {
  FeeTier,
  Liquidity,
  PoolKey,
  SqrtPrice,
  getDeltaY,
  getLiquidityScale,
  getPercentageScale,
  getSqrtPriceScale,
  getTokenAmountScale,
  newFeeTier,
  newPoolKey
} from 'math/math.js'
import { deployInvariant, deployPSP22, getEnvTestAccount } from './testUtils.js'

const main = async () => {
  {
    console.log(getSqrtPriceScale())
    console.log(getTokenAmountScale())
    console.log(getPercentageScale())
    console.log(getLiquidityScale())
  }
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
  const invariant = await deployInvariant(api, account, initFee, network)

  // deploy token
  const token = await deployPSP22(api, account, 1000n, 'Coin', 'COIN', 12n, network)

  // deploy wrapped azero
  const wazero = await WrappedAZERO.getContract(
    api,
    account,
    null,
    DEFAULT_REF_TIME,
    DEFAULT_PROOF_SIZE,
    network
  )

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

  // wrap and unwrap azero
  console.log('balance before deposit: ', await wazero.balanceOf(account, account.address))
  await wazero.deposit(account, 1000000000000)
  console.log('balance after deposit: ', await wazero.balanceOf(account, account.address))
  await wazero.withdraw(account, 1000000000000)
  console.log('balance after withdraw: ', await wazero.balanceOf(account, account.address))

  const results = await Promise.all([
    invariant.getFeeTiers(account),
    token.totalSupply(account),
    wazero.balanceOf(account, account.address)
  ])

  console.log(results)

  // // get events from past 100 blocks
  // const blockNumber = await api.query.system.number()
  // for (let i = 0; i < 100; i++) {
  //   const previousBlockNumber = (blockNumber as unknown as number) - 1 - i
  //   const previousBlockHash = await api.query.system.blockHash(previousBlockNumber)
  //   const apiAt = await api.at(previousBlockHash.toString())
  //   const events = await apiAt.query.system.events()
  //   console.log((events as any).length)
  // }

  process.exit(0)
}

main()
