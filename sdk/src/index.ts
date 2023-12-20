import dotenv from "dotenv";
import { Invariant } from "./invariant.js";
import { getDeploymentData, initPolkadotJs, sleep } from "./utils.js";
dotenv.config();

const main = async () => {
  const { api, account } = await initPolkadotJs();
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
  // Wait for block to finalize
  await sleep(1000);

  console.log("Received txHash  = ", txHash);

  let newFee = await invariant.getProtocolFee();
  console.log(newFee);

  console.log("Passed.");
};

main();
