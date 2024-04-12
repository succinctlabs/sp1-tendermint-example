// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";
import {DummyVerifier} from "../src/DummyVerifier.sol";

contract SP1TendermintTest is Test {
    DummyVerifier public verifier;
    SP1Tendermint public sp1;

    function setUp() public {
        verifier = new DummyVerifier();
        bytes32 trustedBlockHash = 0x41410655235f653628714eecd34b317e60b26ee3eae9127a13c2dd88f0e2a291;
        sp1 = new SP1Tendermint(address(verifier), 10, trustedBlockHash);
    }

    function test_addNewBlockHash() public {
        sp1.update(
            10,
            hex"41410655235f653628714eecd34b317e60b26ee3eae9127a13c2dd88f0e2a2918f988d0d730aef11ae9c4f3cd9adfb3b6aac94a20948f037beeac22f8df362586753756363657373",
            bytes("")
        );
    }
}
