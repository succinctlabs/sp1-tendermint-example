// SPDX-License-Identifier: MIT
pragma solidity >=0.5.0;

import {INetworkVerifier} from "./INetworkVerifier.sol";

// MockGroth16Verifier is a mock implementation of INetworkVerifier that uses a simple hash
// of the public values and the verification key as a proof.
contract MockGroth16Verifier is INetworkVerifier {
    function version() external pure returns (string memory) {
        return "0.1.0";
    }

    uint8 public constant GROTH16_CODE = 2;

    function verify(bytes32 vkDigest, bytes calldata publicValues, bytes calldata proof)
        external
        pure
        override
        returns (bool)
    {
        bytes32 pvDigest = sha256(publicValues);

        bytes32 expectedProof = sha256(abi.encodePacked(GROTH16_CODE, vkDigest, pvDigest));
        return proof.length == 32 && abi.decode(proof, (bytes32)) == expectedProof;
    }
}
