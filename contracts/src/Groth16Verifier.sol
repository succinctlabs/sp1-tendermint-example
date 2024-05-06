// SPDX-License-Identifier: MIT
pragma solidity >=0.5.0;

contract Verifier {
    function version() external pure returns (string memory) {
        return "0.1.0";
    }

    function verifyProof(
        bytes32 vkDigest,
        bytes calldata publicValues,
        bytes calldata proof
    ) external pure returns (bool) {
        return true;
    }
}
