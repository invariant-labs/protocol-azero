import { ApiPromise, Keyring } from '@polkadot/api'
import { IKeyringPair } from '@polkadot/types/types/interfaces'
import { expect } from 'chai'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { getDeploymentData, initPolkadotApi } from '../src/utils'

describe('psp22', function () {
  const init = async (): Promise<{
    api: ApiPromise
    account: IKeyringPair
    testAccount: IKeyringPair
  }> => {
    const api = await initPolkadotApi(Network.Local)

    const keyring = new Keyring({ type: 'sr25519' })
    const account = await keyring.addFromUri('//Alice')
    const testAccount = await keyring.addFromUri('//Bob')

    return { api, account, testAccount }
  }

  it('deploys', async () => {
    const { api, account } = await init()

    const tokenData = await getDeploymentData('psp22')
    const token = new PSP22(api, Network.Local)

    const name = 'Coin'
    const symbol = 'COIN'

    const tokenDeploy = await token.deploy(
      account,
      tokenData.abi,
      tokenData.wasm,
      1000n,
      name,
      symbol,
      12
    )
    await token.load(tokenDeploy.address, tokenData.abi)
  })

  it('should set metadata', async () => {
    const { api, account } = await init()

    const tokenData = await getDeploymentData('psp22')
    const token = new PSP22(api, Network.Local)

    const name = 'Coin'
    const symbol = 'COIN'

    const tokenDeploy = await token.deploy(
      account,
      tokenData.abi,
      tokenData.wasm,
      500n,
      name,
      symbol,
      12
    )
    await token.load(tokenDeploy.address, tokenData.abi)

    expect(await token.tokenName(account)).to.equal('Coin')
    expect(await token.tokenSymbol(account)).to.equal('COIN')
    expect(await token.tokenDecimals(account)).to.equal(12)
  })

  it('should mint tokens', async () => {
    const { api, account } = await init()

    const tokenData = await getDeploymentData('psp22')
    const token = new PSP22(api, Network.Local)

    const name = 'Coin'
    const symbol = 'COIN'

    const tokenDeploy = await token.deploy(
      account,
      tokenData.abi,
      tokenData.wasm,
      500n,
      name,
      symbol,
      12
    )
    await token.load(tokenDeploy.address, tokenData.abi)

    await token.mint(account, 500)
    expect(await token.balanceOf(account, account.address)).to.equal(1000)
  })

  it('should transfer tokens', async () => {
    const { api, account, testAccount } = await init()

    const tokenData = await getDeploymentData('psp22')
    const token = new PSP22(api, Network.Local)

    const name = 'Coin'
    const symbol = 'COIN'

    const tokenDeploy = await token.deploy(
      account,
      tokenData.abi,
      tokenData.wasm,
      500n,
      name,
      symbol,
      12
    )
    await token.load(tokenDeploy.address, tokenData.abi)

    const data = api.createType('Vec<u8>', [])
    await token.transfer(account, testAccount.address, 250, data)
    expect(await token.balanceOf(account, account.address)).to.equal(250)
    expect(await token.balanceOf(account, testAccount.address)).to.equal(250)
  })

  it('should approve tokens', async () => {
    const { api, account, testAccount } = await init()

    const tokenData = await getDeploymentData('psp22')
    const token = new PSP22(api, Network.Local)

    const name = 'Coin'
    const symbol = 'COIN'

    const tokenDeploy = await token.deploy(
      account,
      tokenData.abi,
      tokenData.wasm,
      500n,
      name,
      symbol,
      12
    )
    await token.load(tokenDeploy.address, tokenData.abi)

    await token.approve(account, testAccount.address, 250)
    expect(await token.allowance(account, account.address, testAccount.address)).to.equal(250)
  })
})
