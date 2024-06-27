import {
  Network,
  initPolkadotApi,
  Invariant,
  TESTNET_INVARIANT_ADDRESS,
  parseEvent
} from '@invariant-labs/a0-sdk'

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)
  const invariant = await Invariant.load(api, network, TESTNET_INVARIANT_ADDRESS)

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
          const decoded = invariant.abi.decodeEvent(record as any)
          console.log('Invariant event: ', parseEvent(decoded))
        }
      }
    })
  }
}

main()
