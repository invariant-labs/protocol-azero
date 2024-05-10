import { Keyring } from '@polkadot/api'
import { SubmittableExtrinsic } from '@polkadot/api/promise/types'
import { expect } from 'chai'
import { Network } from '../src/network'
import { PSP22 } from '../src/psp22'
import { initPolkadotApi } from '../src/utils'

const api = await initPolkadotApi(Network.Local)

const keyring = new Keyring({ type: 'sr25519' })
const account = await keyring.addFromUri('//Alice')

let token0Address = await PSP22.deploy(api, account, 1000000000n, 'Coin', 'COIN', 0n)
const psp22 = await PSP22.load(api, Network.Local, token0Address)

describe('tx', function () {
  beforeEach(async () => {
    token0Address = await PSP22.deploy(api, account, 1000n, 'Coin', 'COIN', 12n)
  })

  it('should send tx', async () => {
    const mintTx = psp22.mintTx(500n)

    const hash = await sendTx(mintTx)

    expect(hash).to.not.be.undefined
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
