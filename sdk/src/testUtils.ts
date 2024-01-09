import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { assert } from 'chai'
import { readFile } from 'fs/promises'
import { InvariantError, Percentage, Position } from 'math/math.js'
import { Invariant } from './invariant.js'
import { Network } from './network.js'
import { PSP22 } from './psp22.js'
import { InvariantTx } from './schema.js'
import { DEFAULT_PROOF_SIZE, DEFAULT_REF_TIME } from './utils.js'
import { WrappedAZERO } from './wrapped_azero.js'

export const positionEquals = async (recievedPosition: Position, expectedPosition: Position) => {
  assert.deepEqual(recievedPosition.poolKey, expectedPosition.poolKey)
  assert.deepEqual(recievedPosition.liquidity.v, expectedPosition.liquidity.v)
  assert.deepEqual(recievedPosition.lowerTickIndex, expectedPosition.lowerTickIndex)
  assert.deepEqual(recievedPosition.upperTickIndex, expectedPosition.upperTickIndex)
  assert.deepEqual(recievedPosition.feeGrowthInsideX.v, expectedPosition.feeGrowthInsideX.v)
  assert.deepEqual(recievedPosition.feeGrowthInsideY.v, expectedPosition.feeGrowthInsideY.v)
  assert.deepEqual(recievedPosition.tokensOwedX, expectedPosition.tokensOwedX)
  assert.deepEqual(recievedPosition.tokensOwedY, expectedPosition.tokensOwedY)
}

export const assertThrowsAsync = async (fn: Promise<any>, word?: InvariantError | InvariantTx) => {
  try {
    await fn
  } catch (e: any) {
    if (word) {
      const err = e.toString()
      console.log(err)
      const regex = new RegExp(`${word}$`)
      if (!regex.test(err)) {
        console.log(err)
        throw new Error('Invalid Error message')
      }
    }
    return
  }
  throw new Error('Function did not throw error')
}

export const deployInvariant = async (
  api: ApiPromise,
  account: IKeyringPair,
  initFee: Percentage,
  network: Network
): Promise<Invariant> => {
  return Invariant.getContract(
    api,
    account,
    null,
    DEFAULT_REF_TIME,
    DEFAULT_PROOF_SIZE,
    initFee,
    network
  )
}

export const deployPSP22 = async (
  api: ApiPromise,
  account: IKeyringPair,
  supply: bigint,
  name: string,
  symbol: string,
  decimals: bigint,
  network: Network
): Promise<PSP22> => {
  return PSP22.getContract(
    api,
    network,
    null,
    DEFAULT_REF_TIME,
    DEFAULT_PROOF_SIZE,
    account,
    supply,
    name,
    symbol,
    decimals
  )
}

export const deployWrappedAZERO = async (
  api: ApiPromise,
  account: IKeyringPair,
  network: Network
): Promise<WrappedAZERO> => {
  return WrappedAZERO.getContract(api, account, null, DEFAULT_REF_TIME, DEFAULT_PROOF_SIZE, network)
}

export const getDeploymentData = async (
  contractName: string
): Promise<{ abi: any; wasm: Buffer }> => {
  try {
    const abi = JSON.parse(
      await readFile(`./contracts/${contractName}/${contractName}.json`, 'utf-8')
    )
    const wasm = await readFile(`./contracts/${contractName}/${contractName}.wasm`)

    return { abi, wasm }
  } catch (error) {
    throw new Error(`${contractName}.json or ${contractName}.wasm not found`)
  }
}

export const sleep = async (ms: number) => {
  return await new Promise(resolve => setTimeout(resolve, ms))
}

export const getEnvTestAccount = async (keyring: Keyring): Promise<IKeyringPair> => {
  const accountUri = process.env.TEST_ACCOUNT_URI

  if (!accountUri) {
    throw new Error('invalid account uri')
  }

  return keyring.addFromUri(accountUri)
}
