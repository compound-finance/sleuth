// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

contract Birthday {
    function query(uint256 age) external pure returns (uint256) {
        return age + 1;
    }
}
