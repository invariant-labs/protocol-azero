import { Keyring } from '@polkadot/api'
import dotenv from 'dotenv'
import { Network } from './network.js'
import { getEnvAccount, initPolkadotApi, newFeeTier, newPoolKey, printBalance } from './utils.js'
import { WrappedAZERO } from './wrapped-azero.js'
dotenv.config()

import { getBalance, transferBalance } from '@scio-labs/use-inkathon'
import {
  CreatePositionEvent,
  FeeTier,
  Liquidity,
  PoolKey,
  SqrtPrice,
  TokenAmount,
  calculateAmountDelta,
  getDeltaY,
  getGlobalMaxSqrtPrice,
  getLiquidityByX,
  getLiquidityByY,
  getLiquidityScale,
  getMaxSqrtPrice,
  getMaxTick,
  getMinTick,
  getPercentageScale,
  getSqrtPriceScale,
  getTokenAmountScale
} from 'math/math.js'
import { Invariant } from './invariant.js'
import { PSP22 } from './psp22.js'
import { InvariantEvent } from './schema.js'
import { getEnvTestAccount } from './testUtils.js'

const main = async () => {
  {
    console.log(getMinTick(1), getMaxTick(5))
  }
  {
    console.log(getSqrtPriceScale())
    console.log(getTokenAmountScale())
    console.log(getPercentageScale())
    console.log(getLiquidityScale())
  }
  {
    const sqrtPriceA: SqrtPrice = 234878324943782000000000000n

    const sqrtPriceB: SqrtPrice = 87854456421658000000000000n
    const liquidity: Liquidity = 983983249092n

    const deltaYUp = getDeltaY(sqrtPriceA, sqrtPriceB, liquidity, true)
    const deltaYDown = getDeltaY(sqrtPriceA, sqrtPriceB, liquidity, false)
    console.log(deltaYUp)
    console.log(deltaYDown)
  }
  {
    const providedAmount: TokenAmount = 47600000000n
    const poolSqrtPrice: SqrtPrice = 1000000000000000000000000000n
    const lowerTickIndex = -22000n
    const upperTickIndex = -21000n

    const { l, amount } = getLiquidityByY(
      providedAmount,
      lowerTickIndex,
      upperTickIndex,
      poolSqrtPrice,
      true
    )
    console.log('Liquidity = ', l)
    console.log('Amount = ', amount)
  }
  {
    const providedAmount = 430000n
    const initSqrtPrice: SqrtPrice = 1005012269622000000000000n
    const lowerTickIndex = 80n
    const upperTickIndex = 120n

    const { l, amount } = getLiquidityByX(
      providedAmount,
      lowerTickIndex,
      upperTickIndex,
      initSqrtPrice,
      true
    )
    console.log('Liquidity = ', l)
    console.log('Amount = ', amount)
  }
  {
    const currentTickIndex = 2n
    const currentSqrtPrice: SqrtPrice = 1000140000000000000000000n
    const liquidity: Liquidity = 5000000000000n
    const liquiditySign = true
    const upperTick = 3n
    const lowerTick = 0n
    const [x, y, updateLiquidity] = calculateAmountDelta(
      currentTickIndex,
      currentSqrtPrice,
      liquidity,
      liquiditySign,
      upperTick,
      lowerTick
    )
    console.log('x = ', x)
    console.log('y = ', y)
    console.log('updateLiquidity = ', updateLiquidity)
  }
  {
    const maxTick: bigint = getMaxTick(1n)
    console.log(maxTick)
    const globalMaxSqrtPrice = getGlobalMaxSqrtPrice()
    const maxSqrtPrice = getMaxSqrtPrice(1)
    console.log(globalMaxSqrtPrice)
    console.log(maxSqrtPrice)
  }
  {
    const feeTier: FeeTier = newFeeTier(10n, 55n)
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
  const initFee = 10n
  const invariant = await Invariant.deploy(api, network, account, initFee)

  invariant.on(InvariantEvent.CreatePositionEvent, (event: CreatePositionEvent) => {
    console.log(event)
  })

  // deploy token
  const token0Address = await PSP22.deploy(api, account, 1000n, 'Coin', 'COIN', 12n)
  const token1Address = await PSP22.deploy(api, account, 1000n, 'Coin', 'COIN', 12n)
  const psp22 = await PSP22.load(api, network, token0Address)
  const feeTier = newFeeTier(6000000000n, 10n)

  const poolKey = await newPoolKey(token0Address, token1Address, feeTier)

  await invariant.addFeeTier(account, feeTier)

  await invariant.createPool(account, poolKey, 1000000000000000000000000n, 0n)

  await psp22.setContractAddress(token0Address)
  await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)
  await psp22.setContractAddress(token1Address)
  await psp22.approve(account, invariant.contract.address.toString(), 10000000000n)

  const pool = await invariant.getPool(account, token0Address, token1Address, feeTier)

  await invariant.createPosition(
    account,
    poolKey,
    -10n,
    10n,
    1000000000000n,
    pool.sqrtPrice,
    pool.sqrtPrice
  )

  // deploy wrapped azero
  const wazero = await WrappedAZERO.deploy(api, network, account)

  await transferBalance(api, account, testAccount.address, 1000000000000)
  console.log('account balance: ', (await getBalance(api, account.address)).balanceFormatted)
  console.log(
    'test account balance: ',
    (await getBalance(api, testAccount.address)).balanceFormatted
  )

  await psp22.setContractAddress(token0Address)
  const results = await Promise.all([
    invariant.getFeeTiers(account),
    psp22.totalSupply(account),
    wazero.balanceOf(account, account.address)
  ])

  console.log(results)

  // get events from past 3 blocks
  const blockNumber = await api.query.system.number()
  for (let i = 0; i < 3; i++) {
    const previousBlockNumber = (blockNumber as unknown as number) - 1 - i
    const previousBlockHash = await api.query.system.blockHash(previousBlockNumber)
    const apiAt = await api.at(previousBlockHash.toString())
    const events = await apiAt.query.system.events()
    console.log((events as any).length)
  }

  process.exit(0)
}

main()
