// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";

contract SP1TendermintScript is Script {
    using stdJson for string;

    SP1Tendermint public tendermint;

    function setUp() public {}

    // Deploy the SP1 Tendermint contract with the supplied initialization parameters.
    function run() public returns (address) {
        vm.startBroadcast();

        // Read the initialization parameters for the SP1 Tendermint contract.
        bytes32 vkey = bytes32(vm.envBytes("TENDERMINT_VKEY_HASH"));
        uint64 trustedHeight = uint64(vm.envUint("TRUSTED_HEIGHT"));
        bytes32 trustedHeaderHash = bytes32(vm.envBytes("TRUSTED_HEADER_HASH"));

        // Deployed contract addresses: https://docs.succinct.xyz/docs/verification/onchain/contract-addresses
        address sp1VerifierGateway = address(
            0x3B6041173B80E77f038f3F2C0f9744f04837185e
        );

        tendermint = new SP1Tendermint(
            vkey,
            trustedHeaderHash,
            trustedHeight,
            sp1VerifierGateway
        );
        vm.stopBroadcast();

        return address(tendermint);
    }
}
