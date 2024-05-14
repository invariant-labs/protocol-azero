import { Keyring } from '@polkadot/api'
import wasm from '../src/wasm/pkg/invariant_a0_wasm.js'

const main = async () => {
  console.log(wasm.simulateInvariantSwap([[12n, 18_446_744_073_709_551_615n]], null, null))
}
main()
