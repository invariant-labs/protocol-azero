import {
  Invariant,
  Network,
  TESTNET_INVARIANT_ADDRESS,
  Tickmap,
  getMaxTick,
  getMinTick,
  initPolkadotApi,
  newFeeTier,
  newPoolKey,
  toPercentage
} from '@invariant-labs/a0-sdk'
import dotenv from 'dotenv'

dotenv.config()

// 200 - token0 - token1 - 1ts
// 500 - token0 - token2 - 1ts
// 1000 - token1 - token2 - 1ts
// 10000 - token0 - token1 - 2ts
let tickmap: Tickmap
const TESTNET_TOKEN_0 = '5DDttUjS4rVQgUtWf7wTXUSXzdUJgwM3NGt53TVpPUhQdv2D'
const TESTNET_TOKEN_1 = '5FhDp42hWXiut6G4RxQQXdkWzg2nX7Yy4fdUWERw46womRFg'
const TESTNET_TOKEN_2 = '5Ct3kJxfL9JMSvUFi44nTUr4fb7Zz47rFJyVUBnZY7tfJpnA'
const FEE_TIER_ONE = newFeeTier(toPercentage(1n, 4n), 1n)
const FEE_TIER_TWO = newFeeTier(2n * toPercentage(1n, 4n), 2n)
const POOL_KEY_ONE = newPoolKey(TESTNET_TOKEN_0, TESTNET_TOKEN_1, FEE_TIER_ONE)
const POOL_KEY_TWO = newPoolKey(TESTNET_TOKEN_0, TESTNET_TOKEN_2, FEE_TIER_ONE)
const POOL_KEY_THREE = newPoolKey(TESTNET_TOKEN_1, TESTNET_TOKEN_2, FEE_TIER_ONE)
const POOL_KEY_FOUR = newPoolKey(TESTNET_TOKEN_0, TESTNET_TOKEN_1, FEE_TIER_TWO)

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const invariant = await Invariant.load(api, network, TESTNET_INVARIANT_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  // 200 ticks initialized
  {
    const poolKey = POOL_KEY_ONE
    {
      const timestampBefore = Date.now()
      tickmap = await invariant.getFullTickmap(poolKey)
      const timestampAfter = Date.now()
      console.log(
        'Time to get full tickmap with 200 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
    {
      const timestampBefore = Date.now()
      await invariant.getAllLiquidityTicks(poolKey, tickmap)
      const timestampAfter = Date.now()
      console.log(
        'Time to get all liquidity ticks with 200 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
  }
  // 500 ticks initialized
  {
    const poolKey = POOL_KEY_TWO
    {
      const timestampBefore = Date.now()
      tickmap = await invariant.getFullTickmap(poolKey)
      const timestampAfter = Date.now()
      console.log(
        'Time to get full tickmap with 500 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
    {
      const timestampBefore = Date.now()
      await invariant.getAllLiquidityTicks(poolKey, tickmap)
      const timestampAfter = Date.now()
      console.log(
        'Time to get all liquidity ticks with 500 ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
  }
  // 1k ticks initialized
  {
    const poolKey = POOL_KEY_THREE
    {
      const timestampBefore = Date.now()
      tickmap = await invariant.getFullTickmap(poolKey)
      const timestampAfter = Date.now()
      console.log(
        'Time to get full tickmap with 1k ticks intialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
    {
      const timestampBefore = Date.now()
      await invariant.getAllLiquidityTicks(poolKey, tickmap)
      const timestampAfter = Date.now()
      console.log(
        'Time to get all liquidity ticks with 1k ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
  }
  // 10k ticks initialized
  {
    const poolKey = POOL_KEY_FOUR
    {
      const timestampBefore = Date.now()
      tickmap = await invariant.getFullTickmap(poolKey)
      const timestampAfter = Date.now()
      console.log(
        'Time to get full tickmap with 10k ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
    {
      const timestampBefore = Date.now()
      await invariant.getAllLiquidityTicks(poolKey, tickmap)
      const timestampAfter = Date.now()
      console.log(
        'Time to get all liquidity ticks with 10k ticks initialized:',
        timestampAfter - timestampBefore,
        'ms'
      )
    }
  }

  process.exit(0)
}

const calculateTicks = (amount: bigint, tickSpacing: bigint): bigint[] => {
  const [minTick, maxTick] = [getMinTick(tickSpacing), getMaxTick(tickSpacing)]

  const dx = (maxTick - minTick) / amount

  const ticks = []
  for (let i = 0n; i < amount; i++) {
    const tick = minTick + i * dx
    ticks.push(tick)
  }
  const set = new Set(ticks)

  if (set.size !== ticks.length) {
    console.log('Ticks are not unique')
    return []
  }

  return ticks
}

const pairTicks = (ticks: bigint[]): { lowerTickIndex: bigint; upperTickIndex: bigint }[] => {
  const pairs = []
  for (let i = 0; i < ticks.length / 2; i++) {
    pairs.push({ lowerTickIndex: ticks[i], upperTickIndex: ticks[ticks.length - i - 1] })
  }

  return pairs
}

main()
