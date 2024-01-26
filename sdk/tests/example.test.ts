import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Pool, Position, Tick, getLiquidityByY, toPercentage, toPrice } from 'math/math.js'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import {
  calculateFee,
  calculateSqrtPriceAfterSlippage,
  initPolkadotApi,
  newFeeTier,
  newPoolKey,
  priceToSqrtPrice
} from '../src/utils'
import { WrappedAZERO } from '../src/wrapped-azero'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

const INVARIANT_ADDRESS = (
  await Invariant.deploy(api, Network.Local, account, 0n)
).contract.address.toString()

const TOKEN0_ADDRESS = await PSP22.deploy(
  api,
  account,
  1000000000000000000000000000000n,
  'Coin',
  'COIN',
  12n
)
const TOKEN1_ADDRESS = await PSP22.deploy(
  api,
  account,
  1000000000000000000000000000000n,
  'Coin',
  'COIN',
  12n
)

const WAZERO_ADDRESS = (
  await WrappedAZERO.deploy(api, Network.Local, account)
).contract.address.toString()
const network = Network.Local

describe('sdk guide snippets', async function () {
  it('sdk guide', async () => {
    // load invariant contract
    const invariant = await Invariant.load(api, Network.Local, INVARIANT_ADDRESS)
    // load token contract
    const psp22 = await PSP22.load(api, Network.Local, TOKEN0_ADDRESS)

    // set fee tier, make sure that fee tier with specified parameters exists
    const feeTier = newFeeTier(toPercentage(1n, 2n), 1n) // fee: 0.01 = 1%, tick spacing: 1

    // If the fee tier does not exist, you have to add it
    const isAdded = await invariant.feeTierExist(account, feeTier)
    if (!isAdded) {
      await invariant.addFeeTier(account, feeTier)
    }

    // set initial price of the pool, we set it to 1.00
    // all endpoints only accept sqrt price so we need to convert it before passing it
    const price = toPrice(1n, 0n)
    const initSqrtPrice = priceToSqrtPrice(price)

    // set pool key, make sure that pool with specified parameters does not exists
    const poolKey = newPoolKey(TOKEN0_ADDRESS, TOKEN1_ADDRESS, feeTier)

    const createPoolResult = await invariant.createPool(account, poolKey, initSqrtPrice)

    // print transaction hash
    console.log(createPoolResult.hash)
    // Output: 0x4324eaff0c4da2d5082fa03c2ef0e0138ed60946525952645a9d8c4d50cb5ec2

    // token y has 12 decimals and we want to add 8 actual tokens to our position
    const tokenYAmount = 8n * 10n ** 12n

    // set lower and upper tick indexes, we want to create position in range [-10, 10]
    const lowerTickIndex = -10n
    const upperTickIndex = 10n

    // calculate amount of token x we need to give to create position
    const { amount: tokenXAmount, l: positionLiquidity } = getLiquidityByY(
      tokenYAmount,
      lowerTickIndex,
      upperTickIndex,
      initSqrtPrice,
      true
    )

    // print amount of token x and y we need to give to create position based on parameteres we passed
    console.log(tokenXAmount, tokenYAmount)
    // Output: 7999999999880n 8000000000000n

    // approve transfers of both tokens
    await psp22.setContractAddress(poolKey.tokenX)
    await psp22.approve(account, invariant.contract.address.toString(), tokenXAmount)
    await psp22.setContractAddress(poolKey.tokenY)
    await psp22.approve(account, invariant.contract.address.toString(), tokenYAmount)

    // create position
    const createPositionResult = await invariant.createPosition(
      account,
      poolKey,
      lowerTickIndex,
      upperTickIndex,
      positionLiquidity,
      initSqrtPrice,
      0n
    )
    console.log(createPositionResult.hash) // print transaction hash
    // Output: 0x652108bb36032bc386fec2eef3f483f29970db7bdbdc9a1a340e279abd626ee2

    // we want to swap 6 tokenX
    // token0 has 12 decimals so we need to multiply it by 10^12
    const amount = 6n * 10n ** 12n

    // approve token x transfer
    await psp22.setContractAddress(poolKey.tokenX)
    await psp22.approve(account, invariant.contract.address.toString(), amount)

    // get estimated result of swap
    const quoteResult = await invariant.quote(account, poolKey, true, amount, true)

    // slippage is a price change you are willing to accept,
    // for examples if current price is 1 and your slippage is 1%, then price limit will be 1.01
    const allowedSlippage = toPercentage(1n, 3n) // 0.001 = 0.1%

    // calculate sqrt price limit based on slippage
    const sqrtPriceLimit = calculateSqrtPriceAfterSlippage(
      quoteResult.targetSqrtPrice,
      allowedSlippage,
      false
    )

    const swapResult = await invariant.swap(account, poolKey, true, amount, true, sqrtPriceLimit)
    console.log(swapResult.hash) // print transaction hash
    // Output: 0xd9cdfddb2c783f24a481811f0f9d7037e2f7202907f092986ecd98838db2b3cb

    // query state
    const poolAfter: Pool = await invariant.getPool(
      account,
      TOKEN0_ADDRESS,
      TOKEN1_ADDRESS,
      feeTier
    )
    const positionAfter: Position = await invariant.getPosition(account, account.address, 0n)
    const lowerTickAfter: Tick = await invariant.getTick(
      account,
      poolKey,
      positionAfter.lowerTickIndex
    )
    const upperTickAfter: Tick = await invariant.getTick(
      account,
      poolKey,
      positionAfter.upperTickIndex
    )

    // pools, ticks and positions have many fee growth fields that are used to calculate fees,
    // by doing that off chain we can save gas fees,
    // so in order to see how many tokens you can claim from fees you need to use calculate fee function
    const fees = calculateFee(poolAfter, positionAfter, lowerTickAfter, upperTickAfter)

    // print amount of unclaimed x and y token
    console.log(fees)
    // Output: { x: 59999999999n, y: 0n }

    // specify position id
    const positionId = 0n
    const claimFeeResult = await invariant.claimFee(account, positionId)
    console.log(claimFeeResult.hash) // print transaction hash
    // Output: 0xead1fe084c904e7b1d0df2f3953c74d03cb90756caea46ae1e896c6956460105

    // get balance of a specific token after claiming position fees and print it
    const accountBalance = await psp22.balanceOf(account, account.address)
    console.log(accountBalance)
    // Output: 999999999999999986060000000119n

    const receiver = keyring.addFromUri('//Bob')

    const positionToTransfer = await invariant.getPosition(account, account.address, 0n)
    // Transfer position from account (signer) to receiver
    await invariant.transferPosition(account, 0n, receiver.address)
    const receiverPosition = await invariant.getPosition(receiver, receiver.address, 0n)
    assert.deepEqual(positionToTransfer, receiverPosition)
    console.log(receiverPosition)
    /* Output: Position {
    poolKey: {
        tokenX: '5CfCkzb2YfGcBVVK5b1UNAyNYra7iAmPrPAZ7joeqbTpG77P',
        tokenY: '5FAjg6DMbbFv9zo1QksGt9GtPGu2qwFXG6jYvdXgybrYJkmR',
        feeTier: { fee: 10000000000n, tickSpacing: 1n }
    },
    liquidity: 16004800319759905588483n,
    lowerTickIndex: -10n,
    upperTickIndex: 10n,
    feeGrowthInsideX: 37488752625000000000000n,
    feeGrowthInsideY: 0n,
    lastBlockNumber: 474n,
    tokensOwedX: 0n,
      tokensOwedY: 0n
    }
    */

    // ### retransfer the position back to the original account
    await invariant.transferPosition(receiver, 0n, account.address)
    // ###

    // remove position
    const removePositionResult = await invariant.removePosition(account, positionId)
    console.log(removePositionResult.hash)
    // Output: 0xe90dfeb5420b26c4f0ed2d5a77825a785a7e42106cc45f5a7d08c597f46c1171

    // get balance of a specific token after removing position
    const accountToken0Balance = await psp22.balanceOf(account, account.address)
    await psp22.setContractAddress(TOKEN1_ADDRESS)
    const accountToken1Balance = await psp22.balanceOf(account, account.address)

    // print balances
    console.log(accountToken0Balance, accountToken1Balance)
    // Output: 999999999999999999999999999998n 999999999999999999999999999998n
  })
  it('sdk guide - using wrapped azero', async () => {
    // load wazero contract
    const wazero = await WrappedAZERO.load(api, network, WAZERO_ADDRESS)

    // send AZERO using deposit method
    await wazero.deposit(account, 1000n)

    // you will receive WAZERO token which you can use as any other token,
    // later you can exchange it back to AZERO at 1:1 ratio
    const accountBalance = await wazero.balanceOf(account, account.address)
    console.log(accountBalance)
    // Output: 1000n
  })
  it('sdk guide - using psp22', async () => {
    // deploy token, it will return tokens address
    const TOKEN0_ADDRESS = await PSP22.deploy(api, account, 500n, 'Coin', 'COIN', 12n)

    // load token by passing its address (you can use existing one), it allows you to interact with it
    const psp22 = await PSP22.load(api, Network.Local, TOKEN0_ADDRESS)

    // interact with token 0
    const account0Balance = await psp22.balanceOf(account, account.address)
    console.log(account0Balance)
    // Output: 500n

    // if you want to interact with different token,
    // simply set different contract address
    await psp22.setContractAddress(TOKEN1_ADDRESS)

    // now we can interact with token y
    const account1Balance = await psp22.balanceOf(account, account.address)
    console.log(account1Balance)
    // Output: 999999999999999999999999999998n
  })
})
