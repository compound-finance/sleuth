// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

contract Pair {
    function query() external pure returns (uint256, string memory) {
        return (55, "hello");
    }

    function queryFail() external pure returns (uint256, string memory) {
        revert("bad news");
    }
}
