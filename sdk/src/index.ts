import dotenv from "dotenv";
import { Invariant } from "./invariant.js";
import { getDeploymentData, initPolkadotJs } from "./utils.js";
dotenv.config();

(async function main() {
  const { api, account } = await initPolkadotJs();
  const { abi, wasm } = await getDeploymentData();
  const invariant = new Invariant(api, account, 100000000000, 100000000000);

  let init_fee = { v: 10 };
  await invariant.new(abi, wasm, init_fee);

  let previous_fee = await invariant.getProtocolFee();
  console.log(previous_fee);

  let new_fee_struct = {
    v: 100000000000,
  };
  await invariant.changeProtocolFee(new_fee_struct);

  let new_fee = await invariant.getProtocolFee();
  console.log(new_fee);
})();
