// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {SP1Verifier} from "./SP1Verifier.sol";

contract SP1Tendermint {
    SP1Verifier public verifier;
    bytes32 public tendermintProgramVkey;
    bytes32 public latestHeader;

    constructor(
        bytes32 tendermintProgramVkey,
        address verifier,
        bytes32 initialBlockHash
    ) {
        verifier = SP1Verifier(verifier);
        tendermintProgramVkey = tendermintProgramVkey;
        latestHeader = initialBlockHash;
    }

    function updateProgramHash(bytes32 tendermintProgramVkey) public {
        tendermintProgramVkey = tendermintProgramVkey;
    }

    function updateVerifier(address verifier) public {
        verifier = SP1Verifier(verifier);
    }

    function updateHeader(
        bytes calldata publicValues,
        bytes calldata proof
    ) public {
        // Extract the first 32 bytes of the public values. This is the trusted block hash.
        bytes memory proofTrustedHeader = publicValues[0:32];
        if (bytes32(proofTrustedHeader) != latestHeader) {
            revert("Trusted block hash and public values do not match");
        }

        // Verify the proof with the associated public values.
        assert(
            verifier.verifySP1Proof(tendermintProgramVkey, proof, publicValues)
        );

        /// The next 32 bytes of the public values are the new trusted block hash. Set the latest
        // header to the new header.
        bytes memory newHeader = publicValues[32:64];
        latestHeader = bytes32(newHeader);
    }
}
