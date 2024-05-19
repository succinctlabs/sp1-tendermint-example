// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {SP1Verifier} from "./SP1Verifier.sol";

contract SP1Tendermint is SP1Verifier {
    bytes32 public tendermintProgramVkey;
    bytes32 public latestHeader;

    error InvalidTrustedHeader();

    constructor(bytes32 _tendermintProgramVkey, bytes32 _initialBlockHash) {
        tendermintProgramVkey = _tendermintProgramVkey;
        latestHeader = _initialBlockHash;
    }

    function verifyTendermintProof(
        bytes calldata proof,
        bytes calldata publicValues
    ) public {
        (bytes32 trustedHeaderHash, bytes32 targetHeaderHash) = abi.decode(
            publicValues,
            (bytes32, bytes32)
        );

        if (trustedHeaderHash != latestHeader) {
            revert InvalidTrustedHeader();
        }

        // Verify the proof with the associated public values. This will revert if proof invalid.
        this.verifyProof(tendermintProgramVkey, publicValues, proof);

        latestHeader = targetHeaderHash;
    }
}
