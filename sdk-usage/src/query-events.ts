import { Invariant } from '@invariant-labs/a0-sdk'
import { Network, initPolkadotApi } from '@invariant-labs/a0-sdk'
import { parseEvent } from '@invariant-labs/a0-sdk/target/utils.js'

const main = async () => {
  const api = await initPolkadotApi(Network.Testnet)
  const invariant = await Invariant.load(
    api,
    Network.Testnet,
    '5GAHRxhgJEcadjfGpLgNNb6gN4794GHX6Z9zjq6JfmYkBAXt'
  )
  const abi = invariant.abi

  const blockNumber = await api.query.system.number()
  for (let i = 0; i < 100; i++) {
    const previousBlockNumber = (blockNumber as unknown as number) - 1 - i
    const previousBlockHash = await api.query.system.blockHash(previousBlockNumber)
    const apiAt = await api.at(previousBlockHash.toString())
    const events = (await apiAt.query.system.events()) as any

    events.forEach((record: any) => {
      const { event } = record

      if (api.events.contracts.ContractEmitted.is(event)) {
        const [account_id] = event.data

        if (account_id.toString() === invariant.contract.address.toString()) {
          const decoded = abi.decodeEvent(record as any)
          console.log('Invariant event: ', parseEvent(decoded))
        }
      }
    })
  }
}

main()
