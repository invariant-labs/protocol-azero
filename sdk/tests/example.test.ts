import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import {
  Pool,
  Position,
  Tick,
  getLiquidityByY,
  toPercentage,
  toPrice
} from 'invariant-a0-wasm/invariant_a0_wasm.js'
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
    console.log(createPositionResult.events)

    // we want to swap 6 token0
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
    console.log(swapResult.hash)
    console.log(swapResult.events)

    // query state
    const pool: Pool = await invariant.getPool(account, TOKEN0_ADDRESS, TOKEN1_ADDRESS, feeTier)
    const position: Position = await invariant.getPosition(account, account.address, 0n)
    const lowerTick: Tick = await invariant.getTick(account, poolKey, position.lowerTickIndex)
    const upperTickAfter: Tick = await invariant.getTick(account, poolKey, position.upperTickIndex)

    // check amount of tokens is able to claim
    const fees = calculateFee(pool, position, lowerTick, upperTickAfter)

    // print amount of unclaimed x and y token
    console.log(fees)

    // get balance of a specific token before claiming position fees and print it
    const accountBalanceBeforeClaim = await psp22.balanceOf(account, account.address)
    console.log(accountBalanceBeforeClaim)

    // specify position id
    const positionId = 0n
    // claim fee
    const claimFeeResult = await invariant.claimFee(account, positionId)
    // print transaction hash
    console.log(claimFeeResult.hash)

    // get balance of a specific token after claiming position fees and print it
    const accountBalanceAfterClaim = await psp22.balanceOf(account, account.address)
    console.log(accountBalanceAfterClaim)

    const receiver = keyring.addFromUri('//Bob')

    const positionToTransfer = await invariant.getPosition(account, account.address, 0n)
    // Transfer position from account (signer) to receiver
    await invariant.transferPosition(account, 0n, receiver.address)
    const receiverPosition = await invariant.getPosition(receiver, receiver.address, 0n)
    assert.deepEqual(positionToTransfer, receiverPosition)
    console.log(receiverPosition)

    // ### retransfer the position back to the original account
    await invariant.transferPosition(receiver, 0n, account.address)
    // ###

    // fetch user balances before removal
    const accountToken0BalanceBeforeRemove = await psp22.balanceOf(account, account.address)
    await psp22.setContractAddress(TOKEN1_ADDRESS)
    const accountToken1BalanceBeforeRemove = await psp22.balanceOf(account, account.address)
    console.log(accountToken0BalanceBeforeRemove, accountToken1BalanceBeforeRemove)

    // remove position
    const removePositionResult = await invariant.removePosition(account, positionId)
    console.log(removePositionResult.hash)

    // get balance of a specific token after removing position
    await psp22.setContractAddress(TOKEN0_ADDRESS)
    const accountToken0BalanceAfterRemove = await psp22.balanceOf(account, account.address)
    await psp22.setContractAddress(TOKEN1_ADDRESS)
    const accountToken1BalanceAfterRemove = await psp22.balanceOf(account, account.address)

    // print balances
    console.log(accountToken0BalanceAfterRemove, accountToken1BalanceAfterRemove)
  })
  it('sdk guide - using wrapped azero', async () => {
    // load wazero contract
    const wazero = await WrappedAZERO.load(api, Network.Local, WAZERO_ADDRESS)

    // get balance of account
    const accountBalanceBefore = await wazero.balanceOf(account, account.address)
    console.log(accountBalanceBefore)

    // send AZERO using deposit method
    await wazero.deposit(account, 1000n)

    // you will receive WAZERO token which you can use as any other token,
    // later you can exchange it back to AZERO at 1:1 ratio
    const accountBalanceAfter = await wazero.balanceOf(account, account.address)
    console.log(accountBalanceAfter)
  })
  it('sdk guide - using psp22', async () => {
    // deploy token, it will return tokens address
    const TOKEN0_ADDRESS = await PSP22.deploy(api, account, 500n, 'CoinA', 'ACOIN', 12n)
    const TOKEN1_ADDRESS = await PSP22.deploy(api, account, 500n, 'CoinB', 'BCOIN', 12n)

    // load token by passing its address (you can use existing one), it allows you to interact with it
    const psp22 = await PSP22.load(api, Network.Local, TOKEN0_ADDRESS)

    // interact with token 0
    const account0Balance = await psp22.balanceOf(account, account.address)
    console.log(account0Balance)

    // if you want to interact with different token,
    // simply set different contract address
    await psp22.setContractAddress(TOKEN1_ADDRESS)

    // now we can interact with token y
    const account1Balance = await psp22.balanceOf(account, account.address)
    console.log(account1Balance)

    // fetch token metadata for previously deployed token0
    await psp22.setContractAddress(TOKEN0_ADDRESS)
    const token0Name = await psp22.tokenName(account)
    const token0Symbol = await psp22.tokenSymbol(account)
    const token0Decimals = await psp22.tokenDecimals(account)
    console.log(token0Name, token0Symbol, token0Decimals)

    // load diffrent token and load its metadata
    await psp22.setContractAddress(TOKEN1_ADDRESS)
    const token1Name = await psp22.tokenName(account)
    const token1Symbol = await psp22.tokenSymbol(account)
    const token1Decimals = await psp22.tokenDecimals(account)
    console.log(token1Name, token1Symbol, token1Decimals)
  })
})
