// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Verifier} from "./Groth16Verifier.sol";

contract SP1Verifier is Verifier {
    function verifySP1Proof(
        bytes32 vkeyHash,
        bytes memory proof,
        bytes memory publicValues
    ) public view returns (bool) {
        uint256[8] memory proofArray = abi.decode(proof, (uint256[8]));
        uint256[2] memory publicInputs;
        // Trim to lowest 253 bits
        publicInputs[0] = uint256(vkeyHash) & ((1 << 253) - 1);
        publicInputs[1] = uint256(sha256(publicValues)) & ((1 << 253) - 1);
        // console.log("publicInputs[0]:", publicInputs[0]);
        // console.log("publicInputs[1]:", publicInputs[1]);
        // console.log("publicValues:");
        // console.logBytes(publicValues);
        this.verifyProof(proofArray, publicInputs);
    }
}
