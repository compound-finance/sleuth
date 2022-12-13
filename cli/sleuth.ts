const solc = require('solc');
import { Provider } from '@ethersproject/providers';
import { Contract } from '@ethersproject/contracts';
import { AbiCoder } from '@ethersproject/abi';
import { keccak256 } from '@ethersproject/keccak256';
import { getContractAddress } from '@ethersproject/address';

interface Opts {
  network?: string,
  version?: number
};

const defaultOpts = {
  network: 'mainnet',
  version: 1
};

const sleuthDeployer = process.env['SLEUTH_ADDRESS'] ?? '0x84C3e20985d9E7aEc46F80d2EB52b731D8CC40F8';

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

  async query(q: string) {
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
    if (result.errors) {
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
    let sleuthCtx = new Contract(this.sleuthAddr, ['function query(bytes) public view returns (bytes)'], this.provider);
    let queryResult = await sleuthCtx.query('0x' + b);
    let res = new AbiCoder().decode(queryAbi.outputs, queryResult);
    if (res.length === 1) {
      return res[0]
    } else {
      return res;
    }
  }
}
