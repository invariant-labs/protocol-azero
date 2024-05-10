import { Keyring } from '@polkadot/api'
import { assert, expect } from 'chai'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')
const testAccount = await keyring.addFromUri('//Bob')

let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

describe('psp22', function () {
  beforeEach(async () => {
    token0Address = await PSP22.deploy(api, account, 1000n, 'Coin', 'COIN', 12n)
  })

  it('should set metadata', async () => {
    await psp22.setContractAddress(token0Address)
    expect(await psp22.tokenName(account.address)).to.equal('Coin')
    expect(await psp22.tokenSymbol(account.address)).to.equal('COIN')
    expect(await psp22.tokenDecimals(account.address)).to.equal(12n)
  })

  it('should mint tokens', async () => {
    await psp22.setContractAddress(token0Address)
    await psp22.mint(account, 500n)
    expect(await psp22.balanceOf(account.address, account.address)).to.equal(1500n)
  })

  it('should transfer tokens', async () => {
    const data = api.createType('Vec<u8>', [])
    await psp22.setContractAddress(token0Address)
    await psp22.transfer(account, testAccount.address, 250n, data)
    expect(await psp22.balanceOf(account.address, account.address)).to.equal(750n)
    expect(await psp22.balanceOf(account.address, testAccount.address)).to.equal(250n)
  })

  it('should change instance', async () => {
    const secondTokenAddress = await PSP22.deploy(api, account, 1000n, 'SecondCoin', 'SCOIN', 12n)
    await psp22.setContractAddress(secondTokenAddress)
    const tokenName = await psp22.tokenName(account.address)
    assert.equal(tokenName, 'SecondCoin')
  })

  it('should approve tokens', async () => {
    await psp22.setContractAddress(token0Address)
    await psp22.approve(account, testAccount.address, 250n)
    expect(await psp22.allowance(account.address, account.address, testAccount.address)).to.equal(
      250n
    )
  })
})
