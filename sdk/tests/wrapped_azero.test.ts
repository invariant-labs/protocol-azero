import { Keyring } from '@polkadot/api'
import { expect } from 'chai'
import { Network } from '../src/network'
import { deployWrappedAZERO, initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let wazero = await deployWrappedAZERO(api, account, Network.Local)

describe('wrapped_azero', function () {
  beforeEach(async () => {
    wazero = await deployWrappedAZERO(api, account, Network.Local)
  })

  it('wraps and unwraps azero', async () => {
    await wazero.deposit(account, 1000000000000)
    expect(await wazero.balanceOf(account, account.address)).to.equal(1000000000000)

    await wazero.withdraw(account, 1000000000000)
    expect(await wazero.balanceOf(account, account.address)).to.equal(0)
  })
})
