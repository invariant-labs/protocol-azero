import { ApiPromise } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { WeightV2 } from "@polkadot/types/interfaces";
import { IKeyringPair } from "@polkadot/types/types/interfaces";
import { deployContract } from "@scio-labs/use-inkathon/helpers";

export class Invariant {
  contract: ContractPromise | null = null;
  api: ApiPromise;
  account: IKeyringPair;
  weight: WeightV2;

  constructor(
    api: ApiPromise,
    account: IKeyringPair,
    refTime: number,
    proofSize: number
  ) {
    this.api = api;
    this.account = account;
    this.weight = api.registry.createType("WeightV2", {
      refTime,
      proofSize,
    }) as WeightV2;
  }

  async new(abi: any, wasm: Buffer, fee: { v: number }): Promise<void> {
    const invariant_deployment = await deployContract(
      this.api,
      this.account,
      abi,
      wasm,
      "new",
      [fee]
    );
    this.contract = new ContractPromise(
      this.api,
      abi,
      invariant_deployment.address
    );
  }

  async getProtocolFee(): Promise<void> {
    return new Promise(async (resolve, reject) => {
      if (!this.contract) {
        reject(new Error("contract not deployed"));
        return;
      }

      const { result, output } = await this.contract.query[
        "invariant::getProtocolFee"
      ](this.account.address, {
        gasLimit: this.weight,
        storageDepositLimit: null,
      });

      if (result.isOk && output) {
        resolve(JSON.parse(output.toString()).ok);
      } else {
        reject(new Error(result.asErr.toHuman()?.toString()));
      }
    });
  }

  async changeProtocolFee(fee: { v: number }): Promise<void> {
    return new Promise(async (resolve, reject) => {
      if (!this.contract) {
        reject(new Error("contract not deployed"));
        return;
      }

      const { result, output } = await this.contract.query[
        "invariant::changeProtocolFee"
      ](
        this.account.address,
        {
          gasLimit: this.weight,
          storageDepositLimit: null,
        },
        fee
      );

      if (result.isOk && output) {
        resolve(JSON.parse(output.toString()).ok);
      } else {
        reject(new Error(result.asErr.toHuman()?.toString()));
      }
    });
  }
}
