import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { expect } from 'chai'
import { Network } from '../src/network'
import { getDeploymentData, initPolkadotApi } from '../src/utils'
import { WrappedAZERO } from '../src/wrapped_azero'

describe('wrapped_azero', function () {
  const init = async (): Promise<{ api: ApiPromise; account: IKeyringPair }> => {
    const api = await initPolkadotApi(Network.Local)

    const keyring = new Keyring({ type: 'sr25519' })
    const account = await keyring.addFromUri('//Alice')

    return { api, account }
  }

  it('deploys', async () => {
    const { api, account } = await init()

    const wazeroData = await getDeploymentData('wrapped_azero')
    const wazero = new WrappedAZERO(api, Network.Local)

    const wazeroDeploy = await wazero.deploy(account, wazeroData.abi, wazeroData.wasm)
    await wazero.load(wazeroDeploy.address, wazeroData.abi)
  })

  it('wraps and unwraps azero', async () => {
    const { api, account } = await init()

    const wazeroData = await getDeploymentData('wrapped_azero')
    const wazero = new WrappedAZERO(api, Network.Local)

    const wazeroDeploy = await wazero.deploy(account, wazeroData.abi, wazeroData.wasm)
    await wazero.load(wazeroDeploy.address, wazeroData.abi)

    await wazero.deposit(account, 1000000000000)
    expect(await wazero.balanceOf(account, account.address)).to.equal(1000000000000)

    await wazero.withdraw(account, 1000000000000)
    expect(await wazero.balanceOf(account, account.address)).to.equal(0)
  })
})
