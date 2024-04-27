// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {MockGroth16Verifier} from "../src/MockGroth16Verifier.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";

contract SP1TendermintScript is Script {
    function setUp() public {}

    function run() public returns (address) {
        vm.startBroadcast();

        address verifierAddress = 0x8d669a7DecF7735e7e0D74E2E8aFed094f292DBE;

        bytes32 trustedBlockHash = bytes32(0x1BAACA085AFB1BFC68B5F58323DAD95B7D3F7BFC5368B13418E6ECD542E058BD);
        bytes32 programHash = bytes32(0xa718c92600de3c1afc00095cdc079d71a6625d09f789e139153ea40d623e0964);

        // Deploy SP1Tendermint.
        SP1Tendermint sp1 = new SP1Tendermint(programHash, address(verifierAddress), trustedBlockHash);
        return address(sp1);
    }
}
