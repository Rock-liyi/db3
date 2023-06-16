/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */

import { Contract, Signer, utils } from "ethers";
import type { Provider } from "@ethersproject/providers";
import type {
  IDB3MetaStore,
  IDB3MetaStoreInterface,
} from "../../../contracts/interfaces/IDB3MetaStore";

const _abi = [
  {
    inputs: [
      {
        internalType: "uint64",
        name: "page",
        type: "uint64",
      },
      {
        internalType: "uint64",
        name: "pageSize",
        type: "uint64",
      },
    ],
    name: "getAllNetworkRegistrations",
    outputs: [
      {
        components: [
          {
            internalType: "string",
            name: "rollupNodeUrl",
            type: "string",
          },
          {
            internalType: "string[]",
            name: "indexNodeUrls",
            type: "string[]",
          },
          {
            internalType: "uint64",
            name: "networkId",
            type: "uint64",
          },
          {
            internalType: "address",
            name: "sender",
            type: "address",
          },
          {
            internalType: "bytes",
            name: "latestArweaveTx",
            type: "bytes",
          },
        ],
        internalType: "struct IDB3MetaStore.NetworkRegistration[]",
        name: "",
        type: "tuple[]",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "uint64",
        name: "networkId",
        type: "uint64",
      },
    ],
    name: "getNetworkRegistration",
    outputs: [
      {
        internalType: "string",
        name: "rollupNodeUrl",
        type: "string",
      },
      {
        internalType: "string[]",
        name: "indexNodeUrls",
        type: "string[]",
      },
      {
        internalType: "uint64",
        name: "registrationNetworkId",
        type: "uint64",
      },
      {
        internalType: "address",
        name: "sender",
        type: "address",
      },
      {
        internalType: "bytes",
        name: "latestArweaveTx",
        type: "bytes",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "uint64",
        name: "networkId",
        type: "uint64",
      },
      {
        internalType: "string",
        name: "indexNodeUrl",
        type: "string",
      },
    ],
    name: "registerIndexNode",
    outputs: [
      {
        internalType: "bool",
        name: "success",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "uint64",
        name: "networkId",
        type: "uint64",
      },
      {
        internalType: "string",
        name: "rollupNodeUrl",
        type: "string",
      },
      {
        internalType: "string[]",
        name: "indexNodeUrls",
        type: "string[]",
      },
      {
        internalType: "bytes",
        name: "latestArweaveTx",
        type: "bytes",
      },
    ],
    name: "registerNetwork",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "uint64",
        name: "networkId",
        type: "uint64",
      },
      {
        internalType: "string",
        name: "rollupNodeUrl",
        type: "string",
      },
    ],
    name: "registerRollupNode",
    outputs: [
      {
        internalType: "bool",
        name: "success",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "uint64",
        name: "networkId",
        type: "uint64",
      },
      {
        internalType: "bytes",
        name: "latestArweaveTx",
        type: "bytes",
      },
    ],
    name: "updateRollupSteps",
    outputs: [
      {
        internalType: "bool",
        name: "success",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;

export class IDB3MetaStore__factory {
  static readonly abi = _abi;
  static createInterface(): IDB3MetaStoreInterface {
    return new utils.Interface(_abi) as IDB3MetaStoreInterface;
  }
  static connect(
    address: string,
    signerOrProvider: Signer | Provider
  ): IDB3MetaStore {
    return new Contract(address, _abi, signerOrProvider) as IDB3MetaStore;
  }
}
