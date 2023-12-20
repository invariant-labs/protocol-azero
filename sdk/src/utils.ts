import { ApiPromise, Keyring } from "@polkadot/api";
import { IKeyringPair } from "@polkadot/types/types/interfaces";
import { getSubstrateChain } from "@scio-labs/use-inkathon/chains";
import {
  getBalance,
  initPolkadotJs as initApi,
} from "@scio-labs/use-inkathon/helpers";
import { SubstrateChain } from "@scio-labs/use-inkathon/types";
import { readFile } from "fs/promises";

export const initPolkadotJs = async (): Promise<{
  api: ApiPromise;
  chain: SubstrateChain;
  account: IKeyringPair;
}> => {
  const accountUri = process.env.ACCOUNT_URI;
  const chainId = process.env.CHAIN;
  const chain = getSubstrateChain(chainId);
  if (!chain) throw new Error("chain not found");

  const { api } = await initApi(chain, { noInitWarn: true });

  const network = (await api.rpc.system.chain())?.toString() || "";
  const version = (await api.rpc.system.version())?.toString() || "";
  console.log(`network: ${network} (${version})`);

  const keyring = new Keyring({ type: "sr25519" });
  if (!accountUri) {
    throw new Error("invalid account uti");
  }
  const account = keyring.addFromUri(accountUri);
  const balance = await getBalance(api, account.address);
  console.log(`account: ${account.address} (${balance.balanceFormatted})\n`);

  return { api, chain, account };
};

export const getDeploymentData = async () => {
  try {
    const abi = JSON.parse(await readFile("./metadata/contract.json", "utf-8"));
    const wasm = await readFile("./metadata/contract.wasm");

    return { abi, wasm };
  } catch (error) {
    throw new Error("contract.json or contract.wasm not found");
  }
};
