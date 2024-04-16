// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {INetworkVerifier} from "./INetworkVerifier.sol";

contract SP1Tendermint {
    INetworkVerifier public verifier;
    bytes32 public tendermintProgramHash;
    bytes32 public latestHeader;

    constructor(
        bytes32 _tendermintProgramHash,
        address _verifier,
        bytes32 _initialBlockHash
    ) {
        verifier = INetworkVerifier(_verifier);
        tendermintProgramHash = _tendermintProgramHash;
        latestHeader = _initialBlockHash;
    }

    function updateProgramHash(bytes32 _tendermintProgramHash) public {
        tendermintProgramHash = _tendermintProgramHash;
    }

    function updateVerifier(address _verifier) public {
        verifier = INetworkVerifier(_verifier);
    }

    function update(
        bytes calldata _publicValues,
        bytes calldata _proof
    ) public {
        // Extract the first 32 bytes of the public values.
        bytes memory proofTrustedHeader = _publicValues[0:32];
        if (bytes32(proofTrustedHeader) != latestHeader) {
            revert("Trusted block hash and public values do not match");
        }

        /// @dev In the dummy verifier, the program hash and proof are not used.
        verifier.verify(tendermintProgramHash, _publicValues, _proof);

        /// Once the proof is verified, the next 32 bytes are the new trusted block hash.
        bytes memory proofNewHeader = _publicValues[32:64];
        latestHeader = bytes32(proofNewHeader);
    }
}
