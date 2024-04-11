// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {INetworkVerifier} from "./INetworkVerifier.sol";

contract SP1Tendermint {
    INetworkVerifier verifier;

    bytes32 latestBlockHash;
    uint256 latestHeight;
    mapping(uint256 => bytes32) blockHashes;

    constructor(address _verifier) {
        verifier = INetworkVerifier(_verifier);
    }

    function skip(
        uint256 _trustedHeight,
        bytes32 _trustedBlockHash,
        bytes calldata _publicValues,
        bytes calldata _proof
    ) public {
        if (_trustedHeight > latestHeight) {
            revert("Trusted height is greater than the latest height");
        }

        bytes32 trustedBlockHash = blockHashes[_trustedHeight];
        if (trustedBlockHash != _trustedBlockHash) {
            revert("Trusted block hash is not equal to the trusted height");
        }

        verifier.verify(bytes32(0), _publicValues, _proof);
    }
}
