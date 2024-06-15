// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

contract SP1Tendermint {
    bytes32 public tendermintProgramVkey;
    bytes32 public latestHeader;
    uint64 public latestHeight;
    ISP1Verifier public verifier;

    error InvalidTrustedHeader();

    /// @notice Constructor.
    /// @param _tendermintProgramVkey The public key of the Tendermint program.
    /// @param _initialBlockHash The initial block hash.
    /// @param _initialHeight The initial height.
    /// @param _verifier The address of the SP1Verifier contract.
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

    /// @notice Verifies a Tendermint proof.
    /// @param proof The proof to verify.
    /// @param publicValues The public values to verify the proof against.
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
