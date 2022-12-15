import { Sleuth } from '../sleuth';
import { BigNumber } from '@ethersproject/bignumber';
import { Provider, JsonRpcProvider } from '@ethersproject/providers';
import * as fs from 'fs/promises';
import * as path from 'path';

describe('testing sleuthing', () => {
  let provider: Provider;

  beforeAll(() => {
    provider = new JsonRpcProvider('http://127.0.0.1:8599');
  });

  test('should return the block number', async () => {
    let sleuth = new Sleuth(provider);
    let solidity = await fs.readFile(path.join(__dirname, '../../src/examples/BlockNumber.sol'), 'utf8');
    let res = await sleuth.fetch(Sleuth.querySol<BigNumber>(solidity));
    expect(res.toNumber()).toBe(1);
  });

  test('should return the pair', async () => {
    let sleuth = new Sleuth(provider);
    let solidity = await fs.readFile(path.join(__dirname, '../../src/examples/Pair.sol'), 'utf8');
    let res = await sleuth.fetch(Sleuth.querySol<[BigNumber, string]>(solidity));
    expect(res[0].toNumber()).toBe(55);
    expect(res[1]).toEqual("hello");
  });

  test('should fail invalid', async () => {
    let sleuth = new Sleuth(provider);
    expect(sleuth.query("INSERT INTO users;")).toEqual("55");
  });

  test('should parse sleuth', async () => {
    let sleuth = new Sleuth(provider);
    let q = sleuth.query<{ number: BigNumber }>("SELECT block.number FROM block;");
    let { number } = await sleuth.fetch(q);
    expect(number.toNumber()).toEqual("55");
  });

  test('should parse sleuth too', async () => {
    let sleuth = new Sleuth(provider);
    let q = sleuth.query<[BigNumber, string, BigNumber]>("SELECT block.number, \"dog\", 22 FROM block;");
    let [number, animal, age] = await sleuth.fetch(q);
    expect(number.toNumber()).toEqual(1);
    expect(animal).toEqual("dog");
    expect(age.toNumber()).toEqual(22);
  });

  test.only('including a call', async () => {
    let sleuth = new Sleuth(provider);
    sleuth.addSource("comet", "0xc3d688B66703497DAA19211EEdff47f25384cdc3", ["function totalSupply() returns (uint256)"]);
    let q = sleuth.query<{totalSupply: BigNumber}>("SELECT comet.totalSupply FROM comet;");
    let { totalSupply } = await sleuth.fetch(q);
    expect(totalSupply.toNumber()).toEqual(22);
  });
});
