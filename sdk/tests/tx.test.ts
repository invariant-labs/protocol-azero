import { Keyring } from '@polkadot/api'
import { SubmittableExtrinsic } from '@polkadot/api/promise/types'
import { expect } from 'chai'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi } from '../src/utils'
import { describe, it } from 'mocha'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local)

describe('tx', function () {
  beforeEach(async () => {
    token0Address = await PSP22.deploy(api, account, 1000n, 'Coin', 'COIN', 12n)
  })

  it('should send tx', async () => {
    const balanceBefore = await psp22.balanceOf(account.address, token0Address)

    const mintAmount = 500n
    const mintTx = psp22.mintTx(mintAmount, token0Address)
    const hash = await sendTx(mintTx)

    expect(hash).to.not.be.undefined

    const balanceAfter = await psp22.balanceOf(account.address, token0Address)
    expect(balanceAfter).to.equal(balanceBefore + mintAmount)
  })
})

async function sendTx(tx: SubmittableExtrinsic) {
  return new Promise(async resolve => {
    await tx.signAndSend(account, result => {
      if (result.isCompleted) {
        resolve(result.txHash)
      }
    })
  })
}
