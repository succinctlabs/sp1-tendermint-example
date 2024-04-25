// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {MockGroth16Verifier} from "../src/MockGroth16Verifier.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";

contract SP1TendermintScript is Script {
    function setUp() public {}

    function run() public returns (address) {
        vm.startBroadcast();

        address verifierAddress = 0x0ca721d495B42cb283EA96b9dB2A2b8FdD0f638C;

        bytes32 trustedBlockHash = bytes32(0x1BAACA085AFB1BFC68B5F58323DAD95B7D3F7BFC5368B13418E6ECD542E058BD);
        bytes32 programHash = bytes32(0x92e3c96f52f74a36e35a0b5e85a0b7440713386885b2d910cb8b6529f0b85e64);

        // Deploy SP1Tendermint.
        SP1Tendermint sp1 = new SP1Tendermint(programHash, address(verifierAddress), trustedBlockHash);
        return address(sp1);
    }
}
