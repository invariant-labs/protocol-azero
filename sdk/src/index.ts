import dotenv from "dotenv";
import { Invariant } from "./invariant.js";
import { getDeploymentData, getEnvAccount, initPolkadotJs, printBalance } from "./utils.js";
import { Network } from "./network.js";
import { Keyring } from "@polkadot/api";
dotenv.config();

const main = async () => {
  const network = Network.getFromEnv();
  const api = await initPolkadotJs(network);
  const keyring = new Keyring({ type: "sr25519" });
  const account = await getEnvAccount(keyring);
  await printBalance(api, account)

  const { abi, wasm } = await getDeploymentData();
  const invariant = new Invariant(api, account, 100000000000, 100000000000);

  let initFee = { v: 10 };
  await invariant.new(abi, wasm, initFee);

  let initialFee = await invariant.getProtocolFee();
  console.log(initialFee);

  let newFeeStruct = {
    v: 100,
  };

  console.log(`Changing protocol fee to: ${newFeeStruct.v}`);

  let txHash = await invariant.changeProtocolFee(newFeeStruct);

  console.log("Received txHash  = ", txHash);

  let newFee = await invariant.getProtocolFee();
  console.log(newFee);

  console.log("Passed.");
  process.exit(0);
};

main();
