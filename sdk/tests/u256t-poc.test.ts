import { Keyring } from '@polkadot/api'
import { deployContract } from '@scio-labs/use-inkathon'
import { describe, it } from 'mocha'
import { Network } from '../src/network'
import { getDeploymentData, initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

describe('u256t-poc', async function () {
  it('entrypoint with uint parameter passes', async () => {
    // const invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
    const fee = 10000000000n
    const pocField = [1n, 0n, 0n, 0n]
    const deploymentData = await getDeploymentData('invariant')
    const deploy = await deployContract(
      api,
      account,
      deploymentData.abi,
      deploymentData.wasm,
      'new',
      [fee, pocField]
    )
    console.log(deploy)
  })
})
