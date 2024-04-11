// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {INetworkVerifier} from "./INetworkVerifier.sol";

contract SP1Tendermint {
    INetworkVerifier verifier;

    bytes32 latestBlockHash;
    uint256 latestHeight;
    mapping(uint256 => bytes32) blockHashes;

    constructor(
        address _verifier,
        uint256 _initialHeight,
        bytes32 _initialBlockHash
    ) {
        verifier = INetworkVerifier(_verifier);
        latestHeight = _initialHeight;
        latestBlockHash = _initialBlockHash;
        blockHashes[_initialHeight] = _initialBlockHash;
    }

    function skip(
        uint256 _trustedHeight,
        bytes calldata _publicValues,
        bytes calldata _proof
    ) public {
        if (_trustedHeight > latestHeight) {
            revert("Trusted height is greater than the latest height");
        }

        // Verify the first 32 bytes of the public values match the trusted block hash.
        bytes32 trustedBlockHash = blockHashes[_trustedHeight];
        // Extract the first 32 bytes of the public values.
        bytes memory _publicValues32 = _publicValues[0:32];
        if (trustedBlockHash != bytes32(_publicValues32)) {
            revert("Trusted block hash and public values do not match");
        }

        verifier.verify(trustedBlockHash, _publicValues, _proof);
    }
}
