import { Keyring } from '@polkadot/api'
import { assert, expect } from 'chai'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const testAccount = await keyring.addFromUri('//Bob')

let token = await PSP22.deploy(api, Network.Local, account, 1000n, 'Coin', 'COIN', 12n)
let address = token.contract.address.toString()
describe('psp22', function () {
  beforeEach(async () => {
    token = await PSP22.deploy(api, Network.Local, account, 1000n, 'Coin', 'COIN', 12n)
    address = token.contract.address.toString()
  })

  it('should set metadata', async () => {
    expect(await token.tokenName(account, address)).to.equal('Coin')
    expect(await token.tokenSymbol(account, address)).to.equal('COIN')
    expect(await token.tokenDecimals(account, address)).to.equal(12n)
  })

  it('should mint tokens', async () => {
    await token.mint(account, 500n, address)
    expect(await token.balanceOf(account, account.address, address)).to.equal(1500n)
  })

  it('should transfer tokens', async () => {
    const data = api.createType('Vec<u8>', [])
    await token.transfer(account, testAccount.address, 250n, data, address)
    expect(await token.balanceOf(account, account.address, address)).to.equal(750n)
    expect(await token.balanceOf(account, testAccount.address, address)).to.equal(250n)
  })

  it('should change instance', async () => {
    const secondToken = await PSP22.deploy(
      api,
      Network.Local,
      account,
      1000n,
      'SecondCoin',
      'SCOIN',
      12n
    )
    const tokenName = await token.tokenName(account, secondToken.contract.address.toString())
    const tokenNameAfterChange = await token.tokenName(account)
    assert.equal(tokenName, 'SecondCoin')
    assert.equal(tokenName, tokenNameAfterChange)
  })

  it('should approve tokens', async () => {
    await token.approve(account, testAccount.address, 250n, address)
    expect(await token.allowance(account, account.address, testAccount.address, address)).to.equal(
      250n
    )
  })
})
