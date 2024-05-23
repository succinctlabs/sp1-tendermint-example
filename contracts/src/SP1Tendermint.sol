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

    function decodePublicValues(
        bytes calldata publicValues
    ) public pure returns (bytes32, bytes32, uint64, uint64) {
        require(publicValues.length == 80, "Invalid public values length");

        bytes32 trustedHeaderHash;
        bytes32 targetHeaderHash;
        uint64 trustedHeight;
        uint64 targetHeight;
        assembly {
            trustedHeaderHash := calldataload(add(publicValues.offset, 0x00))
            targetHeaderHash := calldataload(add(publicValues.offset, 0x20))
            trustedHeight := calldataload(add(publicValues.offset, 0x28))
            targetHeight := calldataload(add(publicValues.offset, 0x30))
        }

        trustedHeight = trustedHeight & 0xffffffffffff;
        targetHeight = targetHeight & 0xffffffffffff;

        return (
            trustedHeaderHash,
            targetHeaderHash,
            trustedHeight,
            targetHeight
        );
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
        ) = decodePublicValues(publicValues);

        if (
            trustedHeaderHash != latestHeader && trustedHeight != latestHeight
        ) {
            revert InvalidTrustedHeader();
        }

        // Verify the proof with the associated public values. This will revert if proof invalid.
        // this.verifyProof(tendermintProgramVkey, publicValues, proof);

        latestHeader = targetHeaderHash;
        latestHeight = targetHeight;
    }
}
