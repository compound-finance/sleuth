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

let blockNumberQuery = await Sleuth.querySol(fs.readFileSync('./MyQuery.sol', 'utf8'));
let sleuth = new Sleuth(provider);
let blockNumber = await sleuth.fetch(blockNumberQuery);
```

You can also use pre-compiled contracts (e.g. if you check in the compilation artifacts from solc).

**MyView.ts**
```ts
import { Sleuth } from '@compound-finance/sleuth';

let blockNumberQuery = await Sleuth.querySol(fs.readFileSync('./out/MyQuery.json', 'utf8'));
let sleuth = new Sleuth(provider);
let blockNumber = await sleuth.fetch(blockNumberQuery);
```

## Sleuth Query Language [Experimental]

Sleuth also comes with a full query language, similar to SQL. You can specify contracts and load data from them. This is a WIP and subject to change.

```ts
import { Sleuth } from '@compound-finance/sleuth';

let sleuth = new Sleuth(provider);

// Add a source so the query language knows the shape of the contracts you'll be querying.
sleuth.addSource("comet", "0xc3d688B66703497DAA19211EEdff47f25384cdc3", ["function totalSupply() returns (uint256)"]);

// Build a query
let q = sleuth.query<[ BigNumber ]>("SELECT comet.totalSupply FROM comet;");

// Fetch the data
let [ totalSupply ] = await sleuth.fetch(q);
```

or all in one:

```ts
import { Sleuth } from '@compound-finance/sleuth';

let sleuth = new Sleuth(provider);

console.log(await sleuth.fetchSql(`
  REGISTER CONTRACT comet AT 0xc3d688B66703497DAA19211EEdff47f25384cdc3 WITH INTERFACE ["function totalSupply() returns (uint256)"];
  SELECT comet.totalSupply FROM comet;
`));
```

There's a lot more work in Sleuth Query Language to do, mostly around allowing you to pull in multiple "rows" since that's a core aspect of SQL, but for one-off queries, it's quite fun! 

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

### Parser

There's an early version up and running, which you can use with Sleuth. See [/parser](/parser) for more information.

## License

MIT

Copyright 2022, Compound Labs, Inc. Geoffrey Hayes.
