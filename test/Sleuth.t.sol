// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../src/Sleuth.sol";
import "./examples/BlockNumber.sol";
import "./examples/Pair.sol";

contract SleuthTest is Test {
    function testBlockNumber() public {
        Sleuth sleuth = new Sleuth();
        uint256 number = abi.decode(sleuth.query(type(BlockNumber).creationCode), (uint256));
        assertEq(number, 1);
    }

    function testPair() public {
        Sleuth sleuth = new Sleuth();
        (uint256 x, string memory y) = abi.decode(sleuth.query(type(Pair).creationCode), (uint256, string));
        assertEq(x, 55);
        assertEq(y, "hello");
    }

    function testYul() public {
        Sleuth sleuth = new Sleuth();
        // Hex from Fun.yul/Query.json
        bytes memory yul = hex"33600055601e8060106000396000f3fe60003560e01c632c46b20514601357600080fd5b600160005260206000f3";
        (bool r) = abi.decode(sleuth.query(yul), (bool));
        assertEq(r, true);
    }

    function testPairFail() public {
        Sleuth sleuth = new Sleuth();
        vm.expectRevert("bad news");
        sleuth.query(type(Pair).creationCode, abi.encodeWithSelector(Pair.queryFail.selector));
    }
}
