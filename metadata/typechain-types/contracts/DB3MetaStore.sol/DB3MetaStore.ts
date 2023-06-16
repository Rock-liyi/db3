/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import type {
  BaseContract,
  BigNumber,
  BigNumberish,
  BytesLike,
  CallOverrides,
  ContractTransaction,
  Overrides,
  PopulatedTransaction,
  Signer,
  utils,
} from "ethers";
import type { FunctionFragment, Result } from "@ethersproject/abi";
import type { Listener, Provider } from "@ethersproject/providers";
import type {
  TypedEventFilter,
  TypedEvent,
  TypedListener,
  OnEvent,
  PromiseOrValue,
} from "../../common";

export declare namespace DB3MetaStore {
  export type NetworkRegistrationStruct = {
    rollupNode: PromiseOrValue<string>;
    indexNodes: PromiseOrValue<string>[];
    arTx: PromiseOrValue<BytesLike>;
    sender: PromiseOrValue<string>;
  };

  export type NetworkRegistrationStructOutput = [
    string,
    string[],
    string,
    string
  ] & {
    rollupNode: string;
    indexNodes: string[];
    arTx: string;
    sender: string;
  };
}

export interface DB3MetaStoreInterface extends utils.Interface {
  functions: {
    "getAllNetworkRegistrations(uint64,uint64)": FunctionFragment;
    "getNetworkRegistration(uint64)": FunctionFragment;
    "registerIndexNode(uint64,address)": FunctionFragment;
    "registerNetwork(uint64,address,address[],bytes)": FunctionFragment;
    "registerRollupNode(uint64,address)": FunctionFragment;
    "updateRollupNode(uint64,bytes)": FunctionFragment;
  };

  getFunction(
    nameOrSignatureOrTopic:
      | "getAllNetworkRegistrations"
      | "getNetworkRegistration"
      | "registerIndexNode"
      | "registerNetwork"
      | "registerRollupNode"
      | "updateRollupNode"
  ): FunctionFragment;

  encodeFunctionData(
    functionFragment: "getAllNetworkRegistrations",
    values: [PromiseOrValue<BigNumberish>, PromiseOrValue<BigNumberish>]
  ): string;
  encodeFunctionData(
    functionFragment: "getNetworkRegistration",
    values: [PromiseOrValue<BigNumberish>]
  ): string;
  encodeFunctionData(
    functionFragment: "registerIndexNode",
    values: [PromiseOrValue<BigNumberish>, PromiseOrValue<string>]
  ): string;
  encodeFunctionData(
    functionFragment: "registerNetwork",
    values: [
      PromiseOrValue<BigNumberish>,
      PromiseOrValue<string>,
      PromiseOrValue<string>[],
      PromiseOrValue<BytesLike>
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "registerRollupNode",
    values: [PromiseOrValue<BigNumberish>, PromiseOrValue<string>]
  ): string;
  encodeFunctionData(
    functionFragment: "updateRollupNode",
    values: [PromiseOrValue<BigNumberish>, PromiseOrValue<BytesLike>]
  ): string;

  decodeFunctionResult(
    functionFragment: "getAllNetworkRegistrations",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getNetworkRegistration",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "registerIndexNode",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "registerNetwork",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "registerRollupNode",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "updateRollupNode",
    data: BytesLike
  ): Result;

  events: {};
}

export interface DB3MetaStore extends BaseContract {
  connect(signerOrProvider: Signer | Provider | string): this;
  attach(addressOrName: string): this;
  deployed(): Promise<this>;

  interface: DB3MetaStoreInterface;

  queryFilter<TEvent extends TypedEvent>(
    event: TypedEventFilter<TEvent>,
    fromBlockOrBlockhash?: string | number | undefined,
    toBlock?: string | number | undefined
  ): Promise<Array<TEvent>>;

  listeners<TEvent extends TypedEvent>(
    eventFilter?: TypedEventFilter<TEvent>
  ): Array<TypedListener<TEvent>>;
  listeners(eventName?: string): Array<Listener>;
  removeAllListeners<TEvent extends TypedEvent>(
    eventFilter: TypedEventFilter<TEvent>
  ): this;
  removeAllListeners(eventName?: string): this;
  off: OnEvent<this>;
  on: OnEvent<this>;
  once: OnEvent<this>;
  removeListener: OnEvent<this>;

  functions: {
    getAllNetworkRegistrations(
      page: PromiseOrValue<BigNumberish>,
      pageSize: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<
      [DB3MetaStore.NetworkRegistrationStructOutput[]] & {
        registrations: DB3MetaStore.NetworkRegistrationStructOutput[];
      }
    >;

    getNetworkRegistration(
      networkId: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<
      [string, string[], string, string] & {
        rollupNode: string;
        indexNodes: string[];
        arTxData: string;
        sender: string;
      }
    >;

    registerIndexNode(
      networkId: PromiseOrValue<BigNumberish>,
      indexNodeAddress: PromiseOrValue<string>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<ContractTransaction>;

    registerNetwork(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      indexNodeAddresses: PromiseOrValue<string>[],
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<ContractTransaction>;

    registerRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<ContractTransaction>;

    updateRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<ContractTransaction>;
  };

  getAllNetworkRegistrations(
    page: PromiseOrValue<BigNumberish>,
    pageSize: PromiseOrValue<BigNumberish>,
    overrides?: CallOverrides
  ): Promise<DB3MetaStore.NetworkRegistrationStructOutput[]>;

  getNetworkRegistration(
    networkId: PromiseOrValue<BigNumberish>,
    overrides?: CallOverrides
  ): Promise<
    [string, string[], string, string] & {
      rollupNode: string;
      indexNodes: string[];
      arTxData: string;
      sender: string;
    }
  >;

  registerIndexNode(
    networkId: PromiseOrValue<BigNumberish>,
    indexNodeAddress: PromiseOrValue<string>,
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): Promise<ContractTransaction>;

  registerNetwork(
    networkId: PromiseOrValue<BigNumberish>,
    rollupNodeAddress: PromiseOrValue<string>,
    indexNodeAddresses: PromiseOrValue<string>[],
    arTxData: PromiseOrValue<BytesLike>,
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): Promise<ContractTransaction>;

  registerRollupNode(
    networkId: PromiseOrValue<BigNumberish>,
    rollupNodeAddress: PromiseOrValue<string>,
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): Promise<ContractTransaction>;

  updateRollupNode(
    networkId: PromiseOrValue<BigNumberish>,
    arTxData: PromiseOrValue<BytesLike>,
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): Promise<ContractTransaction>;

  callStatic: {
    getAllNetworkRegistrations(
      page: PromiseOrValue<BigNumberish>,
      pageSize: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<DB3MetaStore.NetworkRegistrationStructOutput[]>;

    getNetworkRegistration(
      networkId: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<
      [string, string[], string, string] & {
        rollupNode: string;
        indexNodes: string[];
        arTxData: string;
        sender: string;
      }
    >;

    registerIndexNode(
      networkId: PromiseOrValue<BigNumberish>,
      indexNodeAddress: PromiseOrValue<string>,
      overrides?: CallOverrides
    ): Promise<boolean>;

    registerNetwork(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      indexNodeAddresses: PromiseOrValue<string>[],
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: CallOverrides
    ): Promise<void>;

    registerRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      overrides?: CallOverrides
    ): Promise<boolean>;

    updateRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: CallOverrides
    ): Promise<boolean>;
  };

  filters: {};

  estimateGas: {
    getAllNetworkRegistrations(
      page: PromiseOrValue<BigNumberish>,
      pageSize: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<BigNumber>;

    getNetworkRegistration(
      networkId: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<BigNumber>;

    registerIndexNode(
      networkId: PromiseOrValue<BigNumberish>,
      indexNodeAddress: PromiseOrValue<string>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<BigNumber>;

    registerNetwork(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      indexNodeAddresses: PromiseOrValue<string>[],
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<BigNumber>;

    registerRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<BigNumber>;

    updateRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<BigNumber>;
  };

  populateTransaction: {
    getAllNetworkRegistrations(
      page: PromiseOrValue<BigNumberish>,
      pageSize: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<PopulatedTransaction>;

    getNetworkRegistration(
      networkId: PromiseOrValue<BigNumberish>,
      overrides?: CallOverrides
    ): Promise<PopulatedTransaction>;

    registerIndexNode(
      networkId: PromiseOrValue<BigNumberish>,
      indexNodeAddress: PromiseOrValue<string>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<PopulatedTransaction>;

    registerNetwork(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      indexNodeAddresses: PromiseOrValue<string>[],
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<PopulatedTransaction>;

    registerRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      rollupNodeAddress: PromiseOrValue<string>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<PopulatedTransaction>;

    updateRollupNode(
      networkId: PromiseOrValue<BigNumberish>,
      arTxData: PromiseOrValue<BytesLike>,
      overrides?: Overrides & { from?: PromiseOrValue<string> }
    ): Promise<PopulatedTransaction>;
  };
}
