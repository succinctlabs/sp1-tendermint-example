// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

// @title SP1Tendermint
// @notice A ZK Tendermint Light Client secured by SP1.
contract SP1Tendermint {
    // @notice The SP1 verification key hash for the Tendermint program.
    bytes32 public tendermintProgramVkeyHash;
    // @notice The latest header hash.
    bytes32 public latestHeader;
    // @notice The latest height.
    uint64 public latestHeight;
    // @notice The SP1 verifier contract.
    ISP1Verifier public verifier;

    error InvalidTrustedHeader();

    // @notice The constructor sets the Tendermint program verification key, the initial block hash, the initial height, and the verifier for SP1 Tendermint proofs.
    // @param _tendermintProgramVkey The verification key for the Tendermint program.
    // @param _initialBlockHash The initial block hash.
    // @param _initialHeight The initial height.
    // @param _verifier The address of the SP1 verifier contract.
    constructor(
        bytes32 _tendermintProgramVkeyHash,
        bytes32 _initialBlockHash,
        uint64 _initialHeight,
        address _verifier
    ) {
        tendermintProgramVkeyHash = _tendermintProgramVkeyHash;
        latestHeader = _initialBlockHash;
        latestHeight = _initialHeight;
        verifier = ISP1Verifier(_verifier);
    }

    // @notice Verify an SP1 Tendermint proof.
    // @param proof The proof to verified. Should correspond to the supplied `publicValues`.
    // @param publicValues The public values to verify the proof against. The `publicValues` is the
    // ABI-encoded tuple: (trustedHeight, targetHeight, trustedHeaderHash, targetHeaderHash)
    function verifyTendermintProof(
        bytes calldata proof,
        bytes calldata publicValues
    ) public {
        (
            uint64 trustedHeight,
            uint64 targetHeight,
            bytes32 trustedHeaderHash,
            bytes32 targetHeaderHash
        ) = abi.decode(publicValues, (uint64, uint64, bytes32, bytes32));

        // If the inputs to the proof don't match the latest header in the contract, don't update
        // the contract state.
        if (
            trustedHeaderHash != latestHeader || trustedHeight != latestHeight
        ) {
            revert InvalidTrustedHeader();
        }

        // Verify the proof with the associated public values.
        verifier.verifyProof(tendermintProgramVkeyHash, publicValues, proof);

        // Update the latest header and height to the new values.
        latestHeader = targetHeaderHash;
        latestHeight = targetHeight;
    }
}
