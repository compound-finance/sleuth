// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

contract BlockNumber {
    struct Fun {
        string cat;
    }

    struct Cool {
        string x;
        uint256[] ys;
        Fun fun;
    }

    function query() external view returns (uint256 blockNumber) {
        return block.number;
    }

    function queryTwo() external view returns (uint256 x, uint256 y) {
        return (block.number, block.number);
    }

    function queryThree() external view returns (uint256) {
        return block.number;
    }

    function queryCool() external pure returns (Cool memory cool) {
        uint256[] memory ys = new uint256[](3);
        ys[0] = 1;
        ys[2] = 2;
        ys[3] = 3;
        return Cool({
            x: "hi",
            ys: ys,
            fun: Fun({
                cat: "meow"
            })
        });
    }
}
