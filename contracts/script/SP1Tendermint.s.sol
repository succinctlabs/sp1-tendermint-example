// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";
import {SP1Verifier} from "@sp1-contracts/SP1Verifier.sol";

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

        SP1Verifier verifier = new SP1Verifier();
        tendermint = new SP1Tendermint(
            vkey,
            trustedHeaderHash,
            trustedHeight,
            address(verifier)
        );
        vm.stopBroadcast();

        return address(tendermint);
    }
}
