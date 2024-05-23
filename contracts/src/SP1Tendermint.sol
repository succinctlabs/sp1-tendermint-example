// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {SP1Verifier} from "./SP1Verifier.sol";

contract SP1Tendermint is SP1Verifier {
    bytes32 public tendermintProgramVkey;
    bytes32 public latestHeader;
    uint64 public latestHeight;

    error InvalidTrustedHeader();

    constructor(
        bytes32 _tendermintProgramVkey,
        bytes32 _initialBlockHash,
        uint64 _initialHeight
    ) {
        tendermintProgramVkey = _tendermintProgramVkey;
        latestHeader = _initialBlockHash;
        latestHeight = _initialHeight;
    }

    function verifyTendermintProof(
        bytes calldata proof,
        bytes calldata publicValues
    ) public {
        (
            bytes32 trustedHeaderHash,
            bytes32 targetHeaderHash,
            uint64 trustedHeight,
            uint64 targetHeight
        ) = abi.decode(publicValues, (bytes32, bytes32, uint64, uint64));

        if (
            trustedHeaderHash != latestHeader && trustedHeight != latestHeight
        ) {
            revert InvalidTrustedHeader();
        }

        // Verify the proof with the associated public values. This will revert if proof invalid.
        this.verifyProof(tendermintProgramVkey, publicValues, proof);

        latestHeader = targetHeaderHash;
        latestHeight = targetHeight;
    }
}
