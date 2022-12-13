# Sleuth

<img src="https://github.com/compound-finance/sleuth/raw/main/logo.png" width="100">

----

Sleuth is an easy way to pull data from an EVM-compatible blockchain, allowing for complex queries, similar to an ethers-multicall. Sleuth works by deploying a smart contract and then invoking it in an `eth_call`. This allows you to use complex logic to pull data from many contracts or other items such as `eth_chainId` or `eth_blockNumber`, which you can use for data analysis or in your Web3 front-end. For example:

**MyQuery.sol** [Note: this is not deployed, and is never deployed]
```sol
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

contract BlockNumber {
    function query() external view returns (uint256) {
        return block.number;
    }
}
```

**MyView.ts**
```ts
import { Sleuth } from '@compound-finance/sleuth';

let sleuth = new Sleuth(provider);

let blockNumber = await sleuth.query(fs.readFileSync('./MyQuery.sol', 'utf8'));
```

## Getting Started

Install Sleuth:

```
yarn add @compound-finance/sleuth

# npm install --save @compound-finance/sleuth
```

Next, simply build a Solidity file and build Sleuth, as above, to execute the query. E.g.

```ts
import { Sleuth } from '@compound-finance/sleuth';

let sleuth = new Sleuth(provider);

let [name, age] = await sleuth.query(`
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

contract SimpleQuery {
    function query() external pure returns (uint256, string memory) {
        return (55, "Bob Jones");
    }
}
`);
```

## Future Considerations

Instead of having users build solidity files, it might be nice to build a proper query language. This could be SQL-like or ORM-style or anything that compiles to say Yul (the intermediate representation used by Solidity). We could then abstract the interface to something interesting, such as:

```ts
await sleuth.query("SELECT comet.name FROM comet(0xc3...) WHERE comet.id = 5");
```

There's so much we could do here and it sounds really fun!

## License

MIT

Copyright 2022, Compound Labs, Inc. Geoffrey Hayes.
