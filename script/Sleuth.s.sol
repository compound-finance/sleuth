// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import "../src/Sleuth.sol";

interface CodeJar {
    function saveCode(bytes memory code) external returns (address);
}

contract Prepare is Script {
    function setUp() public {}

    function run() public returns (address) {
        CodeJar codeJar = CodeJar(vm.envAddress("CODE_JAR"));
        console.log("Code Jar Address:", address(codeJar));
        console.log("Chain ID:", block.chainid);
        console.logBytes(address(codeJar).code);

        address sleuthAddress = codeJar.saveCode(type(Sleuth).creationCode);

        console.log("Sleuth Address:", sleuthAddress);

        return sleuthAddress;
    }
}

contract Deploy is Script {
    error MismatchedSleuthAddress(address expected, address actual);
    function setUp() public {}

    function run() public returns (address) {
        bytes memory sleuthCreationCode = vm.getCode("./.release-tmp/Sleuth.json");
        CodeJar codeJar = CodeJar(vm.envAddress("CODE_JAR"));
        address expectedSleuthAddress = vm.envAddress("SLEUTH_ADDRESS");
        address sleuthAddress = codeJar.saveCode(sleuthCreationCode);
        if (sleuthAddress != expectedSleuthAddress) {
            revert MismatchedSleuthAddress(expectedSleuthAddress, sleuthAddress);
        }
        vm.startBroadcast();
        sleuthAddress = codeJar.saveCode(sleuthCreationCode);
        vm.stopBroadcast();

        console.log("Sleuth Address:", sleuthAddress);

        return sleuthAddress;
    }
}
