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
        sp1 = new SP1Tendermint(address(verifier), 10, bytes32("aa"));
    }

    function test_addNewBlockHash() public {
        sp1.skip(10, bytes(""), bytes(""));
    }
}
