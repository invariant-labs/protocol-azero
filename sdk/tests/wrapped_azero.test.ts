import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { Network } from '../src/network'
import { deployWrappedAZERO } from '../src/testUtils'
import { initPolkadotApi } from '../src/utils'

describe('wrapped_azero', function () {
  const init = async (): Promise<{ api: ApiPromise; account: IKeyringPair }> => {
    const api = await initPolkadotApi(Network.Local)

    const keyring = new Keyring({ type: 'sr25519' })
    const account = await keyring.addFromUri('//Alice')

    return { api, account }
  }

  it('deploys', async () => {
    const { api, account } = await init()
    await deployWrappedAZERO(api, account, Network.Local)
  })

  // it('wraps and unwraps azero', async () => {
  //   const { api, account } = await init()

  //   const wazero = await deployWrappedAZERO(api, account, Network.Local)

  //   await wazero.deposit(account, 1000000000000)
  //   expect(await wazero.balanceOf(account, account.address)).to.equal(1000000000000)

  //   await wazero.withdraw(account, 1000000000000)
  //   expect(await wazero.balanceOf(account, account.address)).to.equal(0)
  // })
})
