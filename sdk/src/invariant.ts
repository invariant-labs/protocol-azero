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
    if (!this.contract) {
      throw new Error("contract not deployed");
    }

    const { result, output } = await this.contract.query[
      "invariant::getProtocolFee"
    ](this.account.address, {
      gasLimit: this.weight,
      storageDepositLimit: null,
    });

    console.log(output?.toHuman());

    if (result.isOk && output) {
      return JSON.parse(output.toString()).ok;
    } else {
      throw new Error(result.asErr.toHuman()?.toString());
    }
  }

  async changeProtocolFee(fee: { v: number }): Promise<string> {
    if (!this.contract) {
      throw new Error("contract not deployed");
    }

    const call = this.contract.tx["invariant::changeProtocolFee"](
      {
        gasLimit: this.weight,
        storageDepositLimit: null,
      },
      fee
    );

    return new Promise<string>(async (resolve, reject) => {
      await call.signAndSend(this.account, (result) => {
        if (result.isFinalized) resolve(result.txHash.toHex());
        else if (result.isError) reject(result.dispatchError);
      });
    });
  }
}
