import { Keyring } from '@polkadot/api'
import { expect } from 'chai'
import { Network } from '../src/network'
import { initPolkadotApi } from '../src/utils'
import { WrappedAZERO } from '../src/wrapped-azero'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let wazero = await WrappedAZERO.deploy(api, Network.Local, account)

describe('wrapped-azero', function () {
  beforeEach(async () => {
    wazero = await WrappedAZERO.deploy(api, Network.Local, account)
  })

  it('wraps and unwraps azero', async () => {
    await wazero.deposit(account, 1000000000000n)
    expect(await wazero.balanceOf(account.address)).to.equal(1000000000000n)

    await wazero.withdraw(account, 1000000000000n)
    expect(await wazero.balanceOf(account.address)).to.equal(0n)
  })
})
