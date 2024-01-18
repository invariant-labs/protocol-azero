import { Keyring } from '@polkadot/api'
import { expect } from 'chai'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const testAccount = await keyring.addFromUri('//Bob')

let token = await PSP22.deploy(api, Network.Local, account, 1000n, 'Coin', 'COIN', 12n)

describe('psp22', function () {
  beforeEach(async () => {
    token = await PSP22.deploy(api, Network.Local, account, 1000n, 'Coin', 'COIN', 12n)
  })

  it('should set metadata', async () => {
    expect(await token.tokenName(account)).to.equal('Coin')
    expect(await token.tokenSymbol(account)).to.equal('COIN')
    expect(await token.tokenDecimals(account)).to.equal(12n)
  })

  it('should mint tokens', async () => {
    await token.mint(account, 500n)
    expect(await token.balanceOf(account, account.address)).to.equal(1500n)
  })

  it('should transfer tokens', async () => {
    const data = api.createType('Vec<u8>', [])
    await token.transfer(account, testAccount.address, 250n, data)
    expect(await token.balanceOf(account, account.address)).to.equal(750n)
    expect(await token.balanceOf(account, testAccount.address)).to.equal(250n)
  })

  it('should approve tokens', async () => {
    await token.approve(account, testAccount.address, 250n)
    expect(await token.allowance(account, account.address, testAccount.address)).to.equal(250n)
  })
})
