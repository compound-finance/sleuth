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

  test('should return the block number via compilation', async () => {
    let sleuth = new Sleuth(provider);
    let solidity = await fs.readFile(path.join(__dirname, '../../src/examples/BlockNumber.sol'), 'utf8');
    let res = await sleuth.fetch(Sleuth.querySol<BigNumber>(solidity));
    expect(res.toNumber()).toBe(1);
  });

  test('should return the block number via precompile', async () => {
    let sleuth = new Sleuth(provider);
    let solidity = await fs.readFile(path.join(__dirname, '../../out/BlockNumber.sol/BlockNumber.json'), 'utf8');
    console.log({solidity})
    let res = await sleuth.fetch(Sleuth.querySol<BigNumber>(solidity));
    console.log("res", res);
    expect(res.toNumber()).toBe(1);
  });

  test('should handle args', async () => {
    let sleuth = new Sleuth(provider);
    let solidity = await fs.readFile(path.join(__dirname, '../../out/Birthday.sol/Birthday.json'), 'utf8');
    console.log({solidity})
    let res = await sleuth.fetch(Sleuth.querySol<BigNumber, [number]>(solidity), [5]);
    console.log("res", res);
    expect(res.toNumber()).toBe(6);
  });

  test('should return the pair', async () => {
    let sleuth = new Sleuth(provider);
    let solidity = await fs.readFile(path.join(__dirname, '../../src/examples/Pair.sol'), 'utf8');
    let res = await sleuth.fetch(Sleuth.querySol<[BigNumber, string]>(solidity));
    console.log(res);
    expect(res[0].toNumber()).toBe(55);
    expect(res[1]).toEqual("hello");
  });

  test('should fail invalid', async () => {
    let sleuth = new Sleuth(provider);
    expect(() => sleuth.query("INSERT INTO users;")).toThrow();
  });

  test('should parse sleuth', async () => {
    let sleuth = new Sleuth(provider);
    let q = sleuth.query<BigNumber>("SELECT block.number FROM block;");
    let number = await sleuth.fetch(q);
    // TODO: Check why named return types aren't working
    expect(number.toNumber()).toEqual(1);
  });

  test('should parse sleuth too', async () => {
    let sleuth = new Sleuth(provider);
    let q = sleuth.query<[BigNumber, string, BigNumber]>("SELECT block.number, \"dog\", 22 FROM block;");
    let [number, animal, age] = await sleuth.fetch(q);
    expect(number.toNumber()).toEqual(1);
    expect(animal).toEqual("dog");
    expect(age.toNumber()).toEqual(22);
  });

  test('including a call', async () => {
    let sleuth = new Sleuth(provider);
    sleuth.addSource("comet", "0xc3d688B66703497DAA19211EEdff47f25384cdc3", ["function totalSupply() returns (uint256)"]);
    let q = sleuth.query<[ BigNumber ]>("SELECT comet.totalSupply FROM comet;");
    let [ totalSupply ] = await sleuth.fetch(q);
    // TODO: Check why named return types aren't working
    expect(totalSupply.toNumber()).toEqual(160);
  });

  test('fetchSql query', async () => {
    let sleuth = new Sleuth(provider);
    let [ totalSupply ] = await sleuth.fetchSql<[ BigNumber ]>(`
      REGISTER CONTRACT comet AT 0xc3d688B66703497DAA19211EEdff47f25384cdc3 WITH INTERFACE ["function totalSupply() returns (uint256)"];
      SELECT comet.totalSupply FROM comet;
    `);
    expect(totalSupply.toNumber()).toEqual(160);
  });
});
