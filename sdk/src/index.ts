import dotenv from "dotenv";
import { Invariant } from "./invariant.js";
import { getDeploymentData, initPolkadotJs } from "./utils.js";
dotenv.config();

const main = async () => {
  const selectedChain = process.argv[2];
  const { api, account } = await initPolkadotJs(selectedChain);
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
};

main();
