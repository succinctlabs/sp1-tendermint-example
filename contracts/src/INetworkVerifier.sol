// SPDX-License-Identifier: MIT
pragma solidity >=0.5.0;

interface INetworkVerifier {
    function version() external returns (string memory);

    function verify(
        bytes32 programHash,
        bytes calldata publicValues,
        bytes calldata proof
    ) external returns (bool);
}
