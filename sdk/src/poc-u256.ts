import { assert } from 'chai'
import { receiveBigType } from './wasm/pkg/invariant_a0_wasm.js'

const main = async () => {
  const u128Max = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
  const u192Max = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
  const u256Max = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
  assert.equal(receiveBigType(u128Max), u128Max)
  assert.equal(receiveBigType(u192Max), u192Max)
  assert.equal(receiveBigType(u256Max), u256Max)
}

main()
