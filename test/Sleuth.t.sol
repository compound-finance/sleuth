// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

import "forge-std/Test.sol";
import "../src/Sleuth.sol";

contract SleuthTest is Test {
    function testBlockNumber() public {
        Sleuth sleuth = new Sleuth();
        bytes memory blockNumber = vm.getCode("BlockNumber.sol");
        uint256 number = abi.decode(sleuth.query(blockNumber), (uint256));
        assertEq(number, 1);
    }

    function testPair() public {
        Sleuth sleuth = new Sleuth();
        bytes memory pair = vm.getCode("Pair.sol");
        (uint256 x, string memory y) = abi.decode(sleuth.query(pair), (uint256, string));
        assertEq(x, 55);
        assertEq(y, "hello");
    }
}
