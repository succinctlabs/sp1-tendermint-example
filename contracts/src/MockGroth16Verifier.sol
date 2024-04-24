// SPDX-License-Identifier: MIT
pragma solidity >=0.5.0;

import {INetworkVerifier} from "./INetworkVerifier.sol";

contract MockGroth16Verifier is INetworkVerifier {
    function version() external pure returns (string memory) {
        return "0.1.0";
    }

    function verify(
        bytes32,
        bytes calldata,
        bytes calldata
    ) external pure override returns (bool) {
        return true;
    }
}
