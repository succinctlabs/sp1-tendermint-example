// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";
import {SP1Verifier} from "../src/SP1Verifier.sol";

contract SP1TendermintScript is Script {
    using stdJson for string;

    SP1Tendermint public tendermint;

    function setUp() public {}

    function run() public returns (address) {
        vm.startBroadcast();
        // Read vkey and trusted header hash from .env
        bytes32 vkey = bytes32(vm.envBytes("VKEY_DIGEST"));
        bytes32 trustedHeaderHash = bytes32(vm.envBytes("TRUSTED_HEADER_HASH"));
        tendermint = new SP1Tendermint(vkey, trustedHeaderHash);
        vm.stopBroadcast();

        return address(tendermint);
    }
}
