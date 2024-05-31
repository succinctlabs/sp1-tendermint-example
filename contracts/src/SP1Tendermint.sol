// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

contract SP1Tendermint {
    bytes32 public tendermintProgramVkey;
    bytes32 public latestHeader;
    uint64 public latestHeight;
    ISP1Verifier public verifier;

    error InvalidTrustedHeader();

    constructor(
        bytes32 _tendermintProgramVkey,
        bytes32 _initialBlockHash,
        uint64 _initialHeight,
        address _verifier
    ) {
        tendermintProgramVkey = _tendermintProgramVkey;
        latestHeader = _initialBlockHash;
        latestHeight = _initialHeight;
        verifier = ISP1Verifier(_verifier);
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
        verifier.verifyProof(tendermintProgramVkey, publicValues, proof);

        latestHeader = targetHeaderHash;
        latestHeight = targetHeight;
    }
}
