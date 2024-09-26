import { Keyring } from '@polkadot/api'
import { Invariant } from '../src/invariant'
import { Network } from '../src/network'
import { initPolkadotApi } from '../src/utils'
import { describe, it } from 'mocha'
import { assert } from 'chai'
import { assertThrowsAsync } from '../src/testUtils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const testAccount = await keyring.addFromUri('//Bob')

let invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)

describe('change-admin', async () => {
  beforeEach(async () => {
    invariant = await Invariant.deploy(api, Network.Local, account, 10000000000n)
  })

  it('change admin works', async () => {
    await invariant.changeAdmin(account, testAccount.address)

    const admin = await invariant.getAdmin()
    assert.deepEqual(admin, testAccount.address)
  })

  it('change admin doesnt work if caller is not an admin', async () => {
    await assertThrowsAsync(invariant.changeAdmin(testAccount, testAccount.address))

    const admin = await invariant.getAdmin()
    assert.deepEqual(admin, account.address)
  })
})
