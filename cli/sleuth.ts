const solc = require('solc');
import { Provider } from '@ethersproject/providers';
import { Contract } from '@ethersproject/contracts';
import { AbiCoder, Fragment, ParamType } from '@ethersproject/abi';
import { keccak256 } from '@ethersproject/keccak256';
import { getContractAddress } from '@ethersproject/address';
import { parse } from '../parser/pkg/parser';

interface Opts {
  network?: string,
  version?: number
};

const defaultOpts = {
  network: 'mainnet',
  version: 1
};

const sleuthDeployer = process.env['SLEUTH_DEPLOYER'] ?? '0x84C3e20985d9E7aEc46F80d2EB52b731D8CC40F8';

interface Query<T> {
  bytecode: string,
  callargs?: string,
  abi: ReadonlyArray<ParamType>
}

export class Sleuth {
  provider: Provider;
  network: string;
  version: number;
  sleuthAddr: string;

  constructor(provider: Provider, opts: Opts = {}) {
    this.provider = provider;
    this.network = opts.network ?? defaultOpts.network;
    this.version = opts.version ?? defaultOpts.version;
    this.sleuthAddr = getContractAddress({ from: sleuthDeployer, nonce: this.version - 1 });
    console.log('Sleuth address', this.sleuthAddr);
  }

  static query<T>(q: string): Query<T> {
    let [tuple, yul] = parse(q).split(';', 2);
    console.log("Tuple", tuple, "Yul", yul);
    const input = {
      language: 'Yul',
      sources: {
        'query.yul': {
          content: yul
        }
      },
      settings: {
        outputSelection: {
          '*': {
            '*': ['*']
          }
        }
      }
    };

    let result = JSON.parse(solc.compile(JSON.stringify(input)));
    console.log(result.contracts['query.yul']);
    if (result.errors && result.errors.length > 0) {
      throw new Error("Compilation Error: " + JSON.stringify(result.errors));
    }
    
    let bytecode = result?.contracts['query.yul']?.Query?.evm?.bytecode?.object;

    if (!bytecode) {
      throw new Error(`Missing bytecode from compilation result: ${JSON.stringify(result)}`);
    }

    return {
      bytecode: bytecode,
      abi: ParamType.from(tuple).components
    };
  }

  static querySol<T>(q: string): Query<T> {
    const input = {
      language: 'Solidity',
      sources: {
        'query.sol': {
          content: q
        }
      },
      settings: {
        outputSelection: {
          '*': {
            '*': ['*']
          }
        }
      }
    };

    let result = JSON.parse(solc.compile(JSON.stringify(input)));
    if (result.errors && result.errors.length > 0) {
      throw new Error("Compilation Error: " + JSON.stringify(result.errors));
    }
    let contract = result.contracts['query.sol'];
    if (!contract) {
      throw new Error(`Missing query.sol compiled contract in ${JSON.stringify(Object.keys(result.contracts))}`);
    }
    let c = Object.values(contract)[0] as any;
    if (!c) {
      throw new Error(`Query does not contain any contract definitions`);
    } else if (Object.keys(contract).length > 1) {
      console.warn(`Query contains multiple contracts, using ${Object.keys(contract)[0]}`);
    }
    let b = c.evm.bytecode.object;
    let abi = c.abi;
    let queryAbi = abi.find(({type, name}: any) => type === 'function' && name === 'query');
    if (!queryAbi) {
      throw new Error(`Query must include function \`query()\``);
    }

    return {
      bytecode: b,
      abi: queryAbi.outputs
    };
  }

  async fetch<T>(q: Query<T>): Promise<T> {
    let sleuthCtx = new Contract(this.sleuthAddr, ['function query(bytes) public view returns (bytes)'], this.provider);
    let queryResult = await sleuthCtx.query('0x' + q.bytecode);
    return new AbiCoder().decode(q.abi, queryResult) as unknown as T;
  }
}
