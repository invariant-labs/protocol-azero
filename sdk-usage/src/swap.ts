import {
  Invariant,
  Keyring,
  Network,
  PSP22,
  TESTNET_INVARIANT_ADDRESS,
  calculateFee,
  initPolkadotApi
} from '@invariant-labs/a0-sdk'
import dotenv from 'dotenv'

dotenv.config()

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)

  const POSITION_ID = 14n
  const SWAP_AMOUNT = 1000000n

  const invariant = await Invariant.load(api, network, TESTNET_INVARIANT_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  const positionBefore = await invariant.getPosition(account.address, POSITION_ID)
  const psp22 = await PSP22.load(api, network, positionBefore.poolKey.tokenX, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)

  await psp22.mint(account, SWAP_AMOUNT)
  await psp22.approve(account, TESTNET_INVARIANT_ADDRESS, SWAP_AMOUNT)

  psp22.setContractAddress(positionBefore.poolKey.tokenY)

  await psp22.mint(account, SWAP_AMOUNT)
  await psp22.approve(account, TESTNET_INVARIANT_ADDRESS, SWAP_AMOUNT)

  await invariant.swap(account, positionBefore.poolKey, true, SWAP_AMOUNT, true, 0n)
  await invariant.swap(account, positionBefore.poolKey, false, SWAP_AMOUNT, true, 2n ** 128n - 1n)

  const pool = await invariant.getPool(
    positionBefore.poolKey.tokenX,
    positionBefore.poolKey.tokenY,
    positionBefore.poolKey.feeTier
  )
  console.log('Pool:', pool)
  const positionAfter = await invariant.getPosition(account.address, POSITION_ID)
  const lowerTick = await invariant.getTick(positionBefore.poolKey, positionBefore.lowerTickIndex)
  const upperTick = await invariant.getTick(positionBefore.poolKey, positionBefore.upperTickIndex)
  console.log('Fees:', calculateFee(pool, positionAfter, lowerTick, upperTick))

  process.exit(0)
}

main()
