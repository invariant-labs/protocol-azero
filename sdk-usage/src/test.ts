import { Invariant, Keyring, Network, PSP22, TESTNET_INVARIANT_ADDRESS, getLiquidityByX, initPolkadotApi, newFeeTier, newPoolKey, sendTx, toPercentage, toSqrtPrice } from "@invariant-labs/a0-sdk"
import dotenv from 'dotenv'

dotenv.config()

// const main = async () => {
//   const network = Network.Testnet
//   const api = await initPolkadotApi(network)

//   const keyring = new Keyring({ type: 'sr25519' })
//   const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
//   const account = keyring.addFromMnemonic(mnemonic)

//   const FEE_TIER = { fee: 200000000n, tickSpacing: 2n }
//   const TOKEN_0_ADDRESS = '5EjKBBJMLE9R2HsXKJRw2CCMZW2q48Ps5bVAQqzsxyhH9jU5'
//   const TOKEN_1_ADDRESS = '5FmDoQPFS5qPMkSumdvVVekiTpsKVmL9E5DHxHEUXCdHFdYy'
//   const POOL_KEY = newPoolKey(TOKEN_0_ADDRESS, TOKEN_1_ADDRESS, FEE_TIER)
//   const AMOUNT = 1000000n
//   console.log(POOL_KEY)

//   const invariant = await Invariant.load(api, network, TESTNET_INVARIANT_ADDRESS, {
//     storageDepositLimit: 100000000000,
//     refTime: 100000000000,
//     proofSize: 100000000000
//   })
//   const psp22 = await PSP22.load(api, network, TOKEN_0_ADDRESS, {
//     storageDepositLimit: 100000000000,
//     refTime: 100000000000,
//     proofSize: 100000000000
//   })

//   console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)

//   await psp22.setContractAddress(TOKEN_0_ADDRESS)
//   await psp22.approve(account, TESTNET_INVARIANT_ADDRESS, AMOUNT)

//   psp22.setContractAddress(TOKEN_1_ADDRESS)

//   await psp22.setContractAddress(TOKEN_1_ADDRESS)
//   await psp22.approve(account, TESTNET_INVARIANT_ADDRESS, AMOUNT)

//   const result = await invariant.createPosition(
//     account,
//     POOL_KEY,
//     -10n,
//     10n,
//     AMOUNT,
//     1000000000000000000000000n,
//     0n
//   )

// console.log(result.hash)

//   process.exit(0)
// }

const main = async () => {
  const network = Network.Testnet
  const api = await initPolkadotApi(network)

  const keyring = new Keyring({ type: 'sr25519' })
  const mnemonic = process.env.DEPLOYER_MNEMONIC ?? ''
  const account = keyring.addFromMnemonic(mnemonic)

  const FEE_TIER = { fee: 200000000n, tickSpacing: 2n }
  const TOKEN_0_ADDRESS = '5EjKBBJMLE9R2HsXKJRw2CCMZW2q48Ps5bVAQqzsxyhH9jU5'
  const TOKEN_1_ADDRESS = '5FmDoQPFS5qPMkSumdvVVekiTpsKVmL9E5DHxHEUXCdHFdYy'
  const POOL_KEY = newPoolKey(TOKEN_0_ADDRESS, TOKEN_1_ADDRESS, FEE_TIER)
  const AMOUNT = 1000000n
  console.log(POOL_KEY)

  const invariant = await Invariant.load(api, network, TESTNET_INVARIANT_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })
  const psp22 = await PSP22.load(api, network, TOKEN_0_ADDRESS, {
    storageDepositLimit: 100000000000,
    refTime: 100000000000,
    proofSize: 100000000000
  })

  console.log(`Deployer: ${account.address}, Uri: ${mnemonic}`)

  await psp22.setContractAddress(TOKEN_0_ADDRESS)
  const XTokenTx =  psp22.approveTx(TESTNET_INVARIANT_ADDRESS,  AMOUNT)
  const TxXResult = await XTokenTx.signAsync(account)
  const XTxid = await sendTx(TxXResult)
  console.log(XTxid.hash)

  await psp22.setContractAddress(TOKEN_1_ADDRESS)
  const YTokenTx = psp22.approveTx( TESTNET_INVARIANT_ADDRESS, AMOUNT)
  const TXYResult = await YTokenTx.signAsync(account)
  const YTxId = await sendTx(TXYResult)
  console.log(YTxId.hash)

  const tx = await invariant.createPositionTx(
    POOL_KEY,
    -10n,
    10n,
    AMOUNT,
    1000000000000000000000000n,
    0n
  )

  const signedTx = await tx.signAsync(account)

  const TxResult = await sendTx(signedTx)

  console.log(TxResult.hash)

  process.exit(0)
}


main()

