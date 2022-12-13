import { Sleuth } from '../sleuth';
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
    let res = await sleuth.query(await fs.readFile(path.join(__dirname, '../../src/examples/BlockNumber.sol'), 'utf8'));
    expect(res.toNumber()).toBe(1);
  });

  test('should return the pair', async () => {
    let sleuth = new Sleuth(provider);
    let res = await sleuth.query(await fs.readFile(path.join(__dirname, '../../src/examples/Pair.sol'), 'utf8'));
    expect(res[0].toNumber()).toBe(55);
    expect(res[1]).toEqual("hello");
  });
});
