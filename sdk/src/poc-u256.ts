import { Keyring } from '@polkadot/api'
import { assert } from 'chai'
import { Invariant } from './invariant.js'
import { Network } from './network.js'
import { initPolkadotApi } from './utils.js'
import { receiveBigType } from './wasm/pkg/invariant_a0_wasm.js'

const main = async () => {
  const u128Max = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
  const u192Max = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
  const u256Max = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
  assert.equal(receiveBigType(u128Max), u128Max)
  assert.equal(receiveBigType(u192Max), u192Max)
  assert.equal(receiveBigType(u256Max), u256Max)

  // Deploy invariant
  const api = await initPolkadotApi(Network.Local)
  const keyring = new Keyring({ type: 'sr25519' })
  const account = keyring.addFromUri('//Alice')

  const invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
  console.log(invariant)
  const protocolFee = await invariant.getProtocolFee()
  console.log(protocolFee)
}

main()
