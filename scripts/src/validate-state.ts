import {
  calculateSqrtPrice,
  Invariant,
  LiquidityTick,
  Network,
  Pool,
  PoolKey,
  PSP22
} from '@invariant-labs/a0-sdk'
import {
  INVARIANT_ADDRESS,
  LIQUIDITY_DENOMINATOR,
  PRICE_DENOMINATOR
} from '@invariant-labs/a0-sdk/target/consts.js'
import { assert, initPolkadotApi } from '@invariant-labs/a0-sdk/target/utils.js'

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const psp22 = await PSP22.load(api, network)

  const invariantAddress = INVARIANT_ADDRESS[network]
  const invariant = await Invariant.load(api, network, invariantAddress)

  const poolKeys = await invariant.getAllPoolKeys()

  const poolPromises: Promise<[PoolKey, Pool, LiquidityTick[]]>[] = []
  for (const poolKey of poolKeys) {
    poolPromises.push(
      new Promise(async resolve => {
        const pool = await invariant.getPool(poolKey.tokenX, poolKey.tokenY, poolKey.feeTier)
        const tickmap = await invariant.getFullTickmap(poolKey)
        const liquidityTicks = await invariant.getAllLiquidityTicks(poolKey, tickmap)

        resolve([poolKey, pool, liquidityTicks])
      })
    )
  }

  const poolsWithTicks = await Promise.all(poolPromises)

  const balances = new Map<string, bigint>()

  for (const [poolKey, pool, liquidityTicks] of poolsWithTicks) {
    const { liquidityX, liquidityY } = getPairLiquidityValues(pool, liquidityTicks)

    const newBalanceX = (balances.get(poolKey.tokenX) ?? 0n) + liquidityX
    const newBalanceY = (balances.get(poolKey.tokenY) ?? 0n) + liquidityY

    balances.set(poolKey.tokenX, newBalanceX)
    balances.set(poolKey.tokenY, newBalanceY)
  }

  const tokens: string[] = []
  balances.forEach((v, k) => tokens.push(k))

  const onchainBalances = await psp22.getAllBalances(tokens, invariantAddress)
  let failed = false

  for (const [token, balance] of balances) {
    const onchainBalance = onchainBalances.get(token)
    if (onchainBalance === undefined || onchainBalance === null) {
      console.error('Failed to fetch balance for', token)
      continue
    }

    const diff = onchainBalance - balance

    if (diff < 0) {
      failed = true
      console.error('Invalid balance', token, balance, onchainBalance, diff)
    }
  }

  process.exit(failed ? 1 : 0)
}

const getPairLiquidityValues = (pool: Pool, liquidityTicks: LiquidityTick[]) => {
  let liquidityX = 0n
  let liquidityY = 0n
  liquidityTicks.sort((a, b) => Number(a.index - b.index))
  const visitedTicks: LiquidityTick[] = []
  for (let i = 0; i < liquidityTicks.length; i++) {
    let curr = liquidityTicks[i]

    if (visitedTicks.length === 0 || curr.sign) {
      visitedTicks.push(curr)
      continue
    }

    for (let j = visitedTicks.length - 1; j >= 0; j--) {
      let prev = visitedTicks[j]

      if (!prev.sign) {
        throw new Error('Prev tick must have positive liquidity')
      }

      let liquidityLower = prev.liquidityChange
      let liquidityUpper = curr.liquidityChange

      let xVal, yVal
      let liquidityDelta
      let lowerTickIndex = prev.index
      let upperTickIndex = curr.index

      if (liquidityUpper >= liquidityLower) {
        liquidityDelta = liquidityLower

        curr.liquidityChange = liquidityUpper - liquidityLower
        visitedTicks.pop()
      } else {
        liquidityDelta = liquidityUpper
        prev.liquidityChange = liquidityLower - liquidityUpper
      }

      const lowerSqrtPrice = calculateSqrtPrice(lowerTickIndex)
      const upperSqrtPrice = calculateSqrtPrice(upperTickIndex)

      try {
        xVal = getX(liquidityDelta, upperSqrtPrice, pool.sqrtPrice, lowerSqrtPrice)
      } catch (error) {
        console.error(error)
        xVal = 0n
      }

      try {
        yVal = getY(liquidityDelta, upperSqrtPrice, pool.sqrtPrice, lowerSqrtPrice)
      } catch (error) {
        console.error(error)
        yVal = 0n
      }
      liquidityX = liquidityX + xVal
      liquidityY = liquidityY + yVal
    }
  }

  assert(visitedTicks.length === 0, 'Ticks were not emptied')

  return { liquidityX, liquidityY }
}

const getX = (
  liquidity: bigint,
  upperSqrtPrice: bigint,
  currentSqrtPrice: bigint,
  lowerSqrtPrice: bigint
): bigint => {
  if (upperSqrtPrice <= 0n || currentSqrtPrice <= 0n || lowerSqrtPrice <= 0n) {
    throw new Error('Price cannot be lower or equal 0')
  }

  let denominator: bigint
  let nominator: bigint

  if (currentSqrtPrice >= upperSqrtPrice) {
    return 0n
  } else if (currentSqrtPrice < lowerSqrtPrice) {
    denominator = (lowerSqrtPrice * upperSqrtPrice) / PRICE_DENOMINATOR
    nominator = upperSqrtPrice - lowerSqrtPrice
  } else {
    denominator = (upperSqrtPrice * currentSqrtPrice) / PRICE_DENOMINATOR
    nominator = upperSqrtPrice - currentSqrtPrice
  }

  return (liquidity * nominator) / denominator / LIQUIDITY_DENOMINATOR
}

export const getY = (
  liquidity: bigint,
  upperSqrtPrice: bigint,
  currentSqrtPrice: bigint,
  lowerSqrtPrice: bigint
): bigint => {
  if (lowerSqrtPrice <= 0n || currentSqrtPrice <= 0n || upperSqrtPrice <= 0n) {
    throw new Error('Price cannot be 0')
  }

  let difference: bigint
  if (currentSqrtPrice <= lowerSqrtPrice) {
    return 0n
  } else if (currentSqrtPrice >= upperSqrtPrice) {
    difference = upperSqrtPrice - lowerSqrtPrice
  } else {
    difference = currentSqrtPrice - lowerSqrtPrice
  }

  return (liquidity * difference) / PRICE_DENOMINATOR / LIQUIDITY_DENOMINATOR
}

main()
