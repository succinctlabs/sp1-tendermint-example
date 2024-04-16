// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {DummyVerifier} from "../src/DummyVerifier.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";

contract SP1TendermintScript is Script {
    function setUp() public {}

    function run() public returns (address) {
        vm.startBroadcast();

        // Deploy dummy verifier.
        address dummyVerifierAddress = 0x7e9EA781E71837b07C7Fb6d4efed938C48a28f51;
        DummyVerifier dummyVerifier = DummyVerifier(dummyVerifierAddress);

        bytes32 trustedBlockHash = bytes32(
            0x41410655235f653628714eecd34b317e60b26ee3eae9127a13c2dd88f0e2a291
        );
        bytes32 programHash = bytes32(0);

        // Deploy SP1Tendermint.
        SP1Tendermint sp1 = new SP1Tendermint(
            programHash,
            address(dummyVerifier),
            trustedBlockHash
        );
        return address(sp1);
    }
}
