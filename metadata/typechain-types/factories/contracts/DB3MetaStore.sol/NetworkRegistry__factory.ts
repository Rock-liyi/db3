/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import { Signer, utils, Contract, ContractFactory, Overrides } from "ethers";
import type { Provider, TransactionRequest } from "@ethersproject/providers";
import type { PromiseOrValue } from "../../../common";
import type {
  NetworkRegistry,
  NetworkRegistryInterface,
} from "../../../contracts/DB3MetaStore.sol/NetworkRegistry";

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
            internalType: "address",
            name: "rollupNode",
            type: "address",
          },
          {
            internalType: "address[]",
            name: "indexNodes",
            type: "address[]",
          },
          {
            internalType: "bytes",
            name: "arTx",
            type: "bytes",
          },
          {
            internalType: "address",
            name: "sender",
            type: "address",
          },
        ],
        internalType: "struct NetworkRegistry.NetworkRegistration[]",
        name: "registrations",
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
        internalType: "address",
        name: "rollupNode",
        type: "address",
      },
      {
        internalType: "address[]",
        name: "indexNodes",
        type: "address[]",
      },
      {
        internalType: "bytes",
        name: "arTxData",
        type: "bytes",
      },
      {
        internalType: "address",
        name: "sender",
        type: "address",
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
        internalType: "address",
        name: "indexNodeAddress",
        type: "address",
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
        internalType: "address",
        name: "rollupNodeAddress",
        type: "address",
      },
      {
        internalType: "address[]",
        name: "indexNodeAddresses",
        type: "address[]",
      },
      {
        internalType: "bytes",
        name: "arTxData",
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
        internalType: "address",
        name: "rollupNodeAddress",
        type: "address",
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
        name: "arTxData",
        type: "bytes",
      },
    ],
    name: "updateRollupNode",
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

const _bytecode =
  "0x608060405234801561001057600080fd5b50611d6c806100206000396000f3fe608060405234801561001057600080fd5b50600436106100625760003560e01c80634f3d2ede14610067578063675d2d381461009a5780636cdc3448146100ca578063c1221c42146100fa578063d8136ad31461012a578063fe3446281461015a575b600080fd5b610081600480360381019061007c9190610e7a565b610176565b6040516100919493929190611036565b60405180910390f35b6100b460048036038101906100af91906110b5565b6103af565b6040516100c19190611110565b60405180910390f35b6100e460048036038101906100df9190611260565b6104bb565b6040516100f19190611110565b60405180910390f35b610114600480360381019061010f91906110b5565b610596565b6040516101219190611110565b60405180910390f35b610144600480360381019061013f91906112bc565b6106c4565b60405161015191906114e1565b60405180910390f35b610174600480360381019061016f91906115cb565b610a7c565b005b60006060806000806000808767ffffffffffffffff1667ffffffffffffffff1681526020019081526020016000209050600073ffffffffffffffffffffffffffffffffffffffff168160000160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1603610239576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610230906116c7565b60405180910390fd5b8060000160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681600101826002018360030160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff168280548060200260200160405190810160405280929190818152602001828054801561030c57602002820191906000526020600020905b8160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190600101908083116102c2575b5050505050925081805461031f90611716565b80601f016020809104026020016040519081016040528092919081815260200182805461034b90611716565b80156103985780601f1061036d57610100808354040283529160200191610398565b820191906000526020600020905b81548152906001019060200180831161037b57829003601f168201915b505050505091509450945094509450509193509193565b6000806000808567ffffffffffffffff1667ffffffffffffffff1681526020019081526020016000209050600073ffffffffffffffffffffffffffffffffffffffff168160000160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff160361046d576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610464906116c7565b60405180910390fd5b828160000160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600191505092915050565b6000806000808567ffffffffffffffff1667ffffffffffffffff1681526020019081526020016000209050600073ffffffffffffffffffffffffffffffffffffffff168160000160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1603610579576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610570906116c7565b60405180910390fd5b8281600201908161058a91906118fd565b50600191505092915050565b6000806000808567ffffffffffffffff1667ffffffffffffffff1681526020019081526020016000209050600073ffffffffffffffffffffffffffffffffffffffff168160000160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1603610654576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161064b906116c7565b60405180910390fd5b80600101839080600181540180825580915050600190039060005260206000200160009091909190916101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550600191505092915050565b60606000826001856106d691906119fe565b6106e09190611a3a565b67ffffffffffffffff16905060008367ffffffffffffffff16826107049190611a77565b90506001548111156107165760015490505b600082826107249190611aab565b90508067ffffffffffffffff8111156107405761073f611135565b5b60405190808252806020026020018201604052801561077957816020015b610766610d2b565b81526020019060019003908161075e5790505b509350600080600190505b6001548167ffffffffffffffff1611610a7157600073ffffffffffffffffffffffffffffffffffffffff166000808367ffffffffffffffff1667ffffffffffffffff16815260200190815260200160002060000160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614610a5e5784821015801561082357508382105b15610a4f576000808267ffffffffffffffff1667ffffffffffffffff1681526020019081526020016000206040518060800160405290816000820160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020016001820180548060200260200160405190810160405280929190818152602001828054801561093457602002820191906000526020600020905b8160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190600101908083116108ea575b5050505050815260200160028201805461094d90611716565b80601f016020809104026020016040519081016040528092919081815260200182805461097990611716565b80156109c65780601f1061099b576101008083540402835291602001916109c6565b820191906000526020600020905b8154815290600101906020018083116109a957829003601f168201915b505050505081526020016003820160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681525050868684610a329190611aab565b81518110610a4357610a42611adf565b5b60200260200101819052505b8180610a5a90611b0e565b9250505b8080610a6990611b56565b915050610784565b505050505092915050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610aeb576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610ae290611bd2565b60405180910390fd5b6000825111610b2f576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610b2690611c3e565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1603610b9e576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610b9590611caa565b60405180910390fd5b60008060008667ffffffffffffffff1667ffffffffffffffff1681526020019081526020016000209050600073ffffffffffffffffffffffffffffffffffffffff168160000160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1614610c5b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610c5290611d16565b60405180910390fd5b838160000160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555082816001019080519060200190610cb6929190610d7f565b5081816002019081610cc891906118fd565b50338160030160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555060016000815480929190610d1f90611b0e565b91905055505050505050565b6040518060800160405280600073ffffffffffffffffffffffffffffffffffffffff1681526020016060815260200160608152602001600073ffffffffffffffffffffffffffffffffffffffff1681525090565b828054828255906000526020600020908101928215610df8579160200282015b82811115610df75782518260006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555091602001919060010190610d9f565b5b509050610e059190610e09565b5090565b5b80821115610e22576000816000905550600101610e0a565b5090565b6000604051905090565b600080fd5b600080fd5b600067ffffffffffffffff82169050919050565b610e5781610e3a565b8114610e6257600080fd5b50565b600081359050610e7481610e4e565b92915050565b600060208284031215610e9057610e8f610e30565b5b6000610e9e84828501610e65565b91505092915050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000610ed282610ea7565b9050919050565b610ee281610ec7565b82525050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b610f1d81610ec7565b82525050565b6000610f2f8383610f14565b60208301905092915050565b6000602082019050919050565b6000610f5382610ee8565b610f5d8185610ef3565b9350610f6883610f04565b8060005b83811015610f99578151610f808882610f23565b9750610f8b83610f3b565b925050600181019050610f6c565b5085935050505092915050565b600081519050919050565b600082825260208201905092915050565b60005b83811015610fe0578082015181840152602081019050610fc5565b60008484015250505050565b6000601f19601f8301169050919050565b600061100882610fa6565b6110128185610fb1565b9350611022818560208601610fc2565b61102b81610fec565b840191505092915050565b600060808201905061104b6000830187610ed9565b818103602083015261105d8186610f48565b905081810360408301526110718185610ffd565b90506110806060830184610ed9565b95945050505050565b61109281610ec7565b811461109d57600080fd5b50565b6000813590506110af81611089565b92915050565b600080604083850312156110cc576110cb610e30565b5b60006110da85828601610e65565b92505060206110eb858286016110a0565b9150509250929050565b60008115159050919050565b61110a816110f5565b82525050565b60006020820190506111256000830184611101565b92915050565b600080fd5b600080fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b61116d82610fec565b810181811067ffffffffffffffff8211171561118c5761118b611135565b5b80604052505050565b600061119f610e26565b90506111ab8282611164565b919050565b600067ffffffffffffffff8211156111cb576111ca611135565b5b6111d482610fec565b9050602081019050919050565b82818337600083830152505050565b60006112036111fe846111b0565b611195565b90508281526020810184848401111561121f5761121e611130565b5b61122a8482856111e1565b509392505050565b600082601f8301126112475761124661112b565b5b81356112578482602086016111f0565b91505092915050565b6000806040838503121561127757611276610e30565b5b600061128585828601610e65565b925050602083013567ffffffffffffffff8111156112a6576112a5610e35565b5b6112b285828601611232565b9150509250929050565b600080604083850312156112d3576112d2610e30565b5b60006112e185828601610e65565b92505060206112f285828601610e65565b9150509250929050565b600081519050919050565b600082825260208201905092915050565b6000819050602082019050919050565b600082825260208201905092915050565b600061134482610ee8565b61134e8185611328565b935061135983610f04565b8060005b8381101561138a5781516113718882610f23565b975061137c83610f3b565b92505060018101905061135d565b5085935050505092915050565b600082825260208201905092915050565b60006113b382610fa6565b6113bd8185611397565b93506113cd818560208601610fc2565b6113d681610fec565b840191505092915050565b60006080830160008301516113f96000860182610f14565b50602083015184820360208601526114118282611339565b9150506040830151848203604086015261142b82826113a8565b91505060608301516114406060860182610f14565b508091505092915050565b600061145783836113e1565b905092915050565b6000602082019050919050565b6000611477826112fc565b6114818185611307565b93508360208202850161149385611318565b8060005b858110156114cf57848403895281516114b0858261144b565b94506114bb8361145f565b925060208a01995050600181019050611497565b50829750879550505050505092915050565b600060208201905081810360008301526114fb818461146c565b905092915050565b600067ffffffffffffffff82111561151e5761151d611135565b5b602082029050602081019050919050565b600080fd5b600061154761154284611503565b611195565b9050808382526020820190506020840283018581111561156a5761156961152f565b5b835b81811015611593578061157f88826110a0565b84526020840193505060208101905061156c565b5050509392505050565b600082601f8301126115b2576115b161112b565b5b81356115c2848260208601611534565b91505092915050565b600080600080608085870312156115e5576115e4610e30565b5b60006115f387828801610e65565b9450506020611604878288016110a0565b935050604085013567ffffffffffffffff81111561162557611624610e35565b5b6116318782880161159d565b925050606085013567ffffffffffffffff81111561165257611651610e35565b5b61165e87828801611232565b91505092959194509250565b600082825260208201905092915050565b7f4e6574776f726b206e6f74207265676973746572656400000000000000000000600082015250565b60006116b160168361166a565b91506116bc8261167b565b602082019050919050565b600060208201905081810360008301526116e0816116a4565b9050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b6000600282049050600182168061172e57607f821691505b602082108103611741576117406116e7565b5b50919050565b60008190508160005260206000209050919050565b60006020601f8301049050919050565b600082821b905092915050565b6000600883026117a97fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8261176c565b6117b3868361176c565b95508019841693508086168417925050509392505050565b6000819050919050565b6000819050919050565b60006117fa6117f56117f0846117cb565b6117d5565b6117cb565b9050919050565b6000819050919050565b611814836117df565b61182861182082611801565b848454611779565b825550505050565b600090565b61183d611830565b61184881848461180b565b505050565b5b8181101561186c57611861600082611835565b60018101905061184e565b5050565b601f8211156118b15761188281611747565b61188b8461175c565b8101602085101561189a578190505b6118ae6118a68561175c565b83018261184d565b50505b505050565b600082821c905092915050565b60006118d4600019846008026118b6565b1980831691505092915050565b60006118ed83836118c3565b9150826002028217905092915050565b61190682610fa6565b67ffffffffffffffff81111561191f5761191e611135565b5b6119298254611716565b611934828285611870565b600060209050601f8311600181146119675760008415611955578287015190505b61195f85826118e1565b8655506119c7565b601f19841661197586611747565b60005b8281101561199d57848901518255600182019150602085019450602081019050611978565b868310156119ba57848901516119b6601f8916826118c3565b8355505b6001600288020188555050505b505050505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6000611a0982610e3a565b9150611a1483610e3a565b9250828203905067ffffffffffffffff811115611a3457611a336119cf565b5b92915050565b6000611a4582610e3a565b9150611a5083610e3a565b9250828202611a5e81610e3a565b9150808214611a7057611a6f6119cf565b5b5092915050565b6000611a82826117cb565b9150611a8d836117cb565b9250828201905080821115611aa557611aa46119cf565b5b92915050565b6000611ab6826117cb565b9150611ac1836117cb565b9250828203905081811115611ad957611ad86119cf565b5b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b6000611b19826117cb565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8203611b4b57611b4a6119cf565b5b600182019050919050565b6000611b6182610e3a565b915067ffffffffffffffff8203611b7b57611b7a6119cf565b5b600182019050919050565b7f496e76616c696420526f6c6c7570206e6f646520616464726573730000000000600082015250565b6000611bbc601b8361166a565b9150611bc782611b86565b602082019050919050565b60006020820190508181036000830152611beb81611baf565b9050919050565b7f4174206c65617374206f6e6520496e646578206e6f6465207265717569726564600082015250565b6000611c2860208361166a565b9150611c3382611bf2565b602082019050919050565b60006020820190508181036000830152611c5781611c1b565b9050919050565b7f496e76616c69642073656e646572206164647265737300000000000000000000600082015250565b6000611c9460168361166a565b9150611c9f82611c5e565b602082019050919050565b60006020820190508181036000830152611cc381611c87565b9050919050565b7f4e6574776f726b20616c72656164792072656769737465726564000000000000600082015250565b6000611d00601a8361166a565b9150611d0b82611cca565b602082019050919050565b60006020820190508181036000830152611d2f81611cf3565b905091905056fea26469706673582212204b150de938f3f1bbd4813171c93ebeef9dd23c3e866513906e0cebf77c4d59da64736f6c63430008110033";

type NetworkRegistryConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: NetworkRegistryConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class NetworkRegistry__factory extends ContractFactory {
  constructor(...args: NetworkRegistryConstructorParams) {
    if (isSuperArgs(args)) {
      super(...args);
    } else {
      super(_abi, _bytecode, args[0]);
    }
  }

  override deploy(
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): Promise<NetworkRegistry> {
    return super.deploy(overrides || {}) as Promise<NetworkRegistry>;
  }
  override getDeployTransaction(
    overrides?: Overrides & { from?: PromiseOrValue<string> }
  ): TransactionRequest {
    return super.getDeployTransaction(overrides || {});
  }
  override attach(address: string): NetworkRegistry {
    return super.attach(address) as NetworkRegistry;
  }
  override connect(signer: Signer): NetworkRegistry__factory {
    return super.connect(signer) as NetworkRegistry__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): NetworkRegistryInterface {
    return new utils.Interface(_abi) as NetworkRegistryInterface;
  }
  static connect(
    address: string,
    signerOrProvider: Signer | Provider
  ): NetworkRegistry {
    return new Contract(address, _abi, signerOrProvider) as NetworkRegistry;
  }
}
