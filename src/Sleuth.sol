// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.23;

contract Sleuth {
    error DeploymentError();

    function query(bytes calldata sleuthQuery) external returns (bytes memory) {
        return queryInternal(sleuthQuery, abi.encodeWithSignature("query()"));
    }

    function query(bytes calldata sleuthQuery, bytes memory calldata_) external returns (bytes memory) {
        return queryInternal(sleuthQuery, calldata_);
    }

    function queryInternal(bytes memory sleuthQuery, bytes memory calldata_) internal returns (bytes memory) {
        address sleuthContract;
        assembly {
            sleuthContract := create(0, add(sleuthQuery, 0x20), mload(sleuthQuery))
        }
        if (sleuthContract == address(0)) {
            revert DeploymentError();
        }

        bool success;
        uint256 retSize;

        assembly {
            success := call(gas(), sleuthContract, 0, add(calldata_, 0x20), mload(calldata_), 0, 0)
            retSize := returndatasize()
        }

        bytes memory sleuthResult = new bytes(retSize);
        assembly {
            returndatacopy(add(sleuthResult, 0x20), 0x00, retSize)
        }

        if (!success) {
            assembly {
                revert(add(sleuthResult, 0x20), retSize)
            }
        }

        return sleuthResult;
    }
}
