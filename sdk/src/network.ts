export enum Network {
  Local = 'local',
  Testnet = 'testnet'
}

export namespace Network {
  export function valueOf(str: string): Network {
    const capitalize = str[0].toUpperCase() + str.slice(1)
    return Network[capitalize as keyof typeof Network] as Network
  }

  export function getFromCli(): Network {
    const network = Network.valueOf(process.argv[2])
    if (!network) {
      throw new Error('network not found')
    }
    return network
  }
}
