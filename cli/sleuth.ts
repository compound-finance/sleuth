import { Provider } from '@ethersproject/providers';
import { Contract } from '@ethersproject/contracts';
import { AbiCoder, FormatTypes, FunctionFragment, Fragment, Interface, ParamType } from '@ethersproject/abi';
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

interface Source {
  name: string,
  address: string,
  iface: Interface
}

interface SolidityQueryOpts {
  queryFunctionName?: string;
}

interface SolcInput {
  language?: string,
  sources: {
    [fileName: string]: {
      content: string
    }
  },
  settings: object
}

interface SolcContract {
  evm?: {
    bytecode?: {
      object: string
    }
  },
  bytecode?: {
    object: string
  },
  abi: Fragment[]
}

interface SolcOutput {
  contracts: {
    [fileName: string]: {
      [contractName: string]: SolcContract
    }
  },
  errors?: string[],
}

function solcCompile(input: SolcInput): SolcOutput {
  let solc;
  try {
    solc = require('solc');
  } catch (e) {
    throw new Error(`solc.js yarn dependency not found. Please build with optional dependencies included`);
  }
  return JSON.parse(solc.compile(JSON.stringify(input)));
}

function hexify(v: string): string {
  return v.startsWith('0x') ? v : `0x${v}`;
}

export class Sleuth {
  provider: Provider;
  network: string;
  version: number;
  sleuthAddr: string;
  sources: Source[];

  constructor(provider: Provider, opts: Opts = {}) {
    this.provider = provider;
    this.network = opts.network ?? defaultOpts.network;
    this.version = opts.version ?? defaultOpts.version;
    this.sleuthAddr = getContractAddress({ from: sleuthDeployer, nonce: this.version - 1 });
    this.sources = [];
  }

  query<T>(q: string): Query<T> {
    let registrations = this.sources.map((source) => {
      let iface = JSON.stringify(source.iface.format(FormatTypes.full));
      return `REGISTER CONTRACT ${source.name} AT ${source.address} WITH INTERFACE ${iface};`
    }).join("\n");
    let fullQuery = `${registrations}${q}`;
    console.log("Full Query", fullQuery);
    let [tuple, yul] = parse(fullQuery).split(';', 2);
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

    let result = solcCompile(input);
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

  static querySol<T>(q: string | object, opts: SolidityQueryOpts = {}): Query<T> {
    if (typeof(q) === 'string') {
      let r;
      try {
        // Try to parse as JSON, if that fails, then consider a query
        r = JSON.parse(q);
      } catch (e) {
        // Ignore
      }

      if (r) {
        return this.querySolOutput(r, opts);
      } else {
        // This must be a source file, try to compile
        return this.querySolSource(q, opts);
      }

    } else {
      // This was passed in as a pre-parsed contract. Or at least, it should have been.
      return this.querySolOutput(q as SolcContract, opts);
    }
  }

  static querySolOutput<T>(c: SolcContract, opts: SolidityQueryOpts = {}): Query<T> {
    let queryFunctionName = opts.queryFunctionName ?? 'query';
    let b = c.evm?.bytecode?.object ?? c.bytecode?.object;
    if (!b) {
      throw new Error(`Missing (evm.)bytecode.object in contract ${JSON.stringify(c, null, 4)}`);
    }
    let abi = c.abi;
    let queryAbi = abi.find(({type, name}: any) => type === 'function' && name === queryFunctionName);
    if (!queryAbi) {
      throw new Error(`Query must include function \`${queryFunctionName}()\``);
    }

    return {
      bytecode: b,
      abi: (queryAbi as FunctionFragment).outputs ?? []
    };
  }

  static querySolSource<T>(q: string, opts: SolidityQueryOpts = {}): Query<T> {
    let fnName = opts.queryFunctionName ?? 'query';
    let input = {
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

    let result = solcCompile(input);
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
    return this.querySolOutput(c, opts);
  }

  async addSource(name: string, address: string, iface: string[] | Interface) {
    if (Array.isArray(iface)) {
      iface = new Interface(iface);
    }
    this.sources.push({name, address, iface});
  }

  async fetch<T>(q: Query<T>): Promise<T> {
    let sleuthCtx = new Contract(this.sleuthAddr, ['function query(bytes) public view returns (bytes)'], this.provider);
    let queryResult = await sleuthCtx.query(hexify(q.bytecode));
    console.log(q.abi);
    console.log(queryResult);
    let r = new AbiCoder().decode(q.abi, queryResult) as unknown;
    if (Array.isArray(r) && r.length === 1) {
      return r[0] as T;
    } else {
      return r as T;
    }
  }

  async fetchSql<T>(q: string): Promise<T> {
    let query = this.query<T>(q);
    return this.fetch<T>(query);
  }
}
