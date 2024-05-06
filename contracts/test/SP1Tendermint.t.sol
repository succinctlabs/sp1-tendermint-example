// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";
import {SP1Verifier} from "../src/SP1Verifier.sol";

contract SP1TendermintTest is Test {
    SP1Verifier public verifier;
    bytes32 public programHash;

    function setUp() public {
        verifier = new SP1Verifier();
        programHash = bytes32(0);
    }

    // Helper function to bytes32 from bytes memory (useful for reading 32 bytes from pv).
    function readBytes(
        bytes calldata bs,
        uint256 start,
        uint256 end
    ) public pure returns (bytes memory) {
        return bs[start:end];
    }

    // Read from fixture.
    function test_fixtureTest() public {
        // Parse JSON {pv, proof} from fixture as bytes.
        string memory fixture = vm.readFile("./fixtures/fixture_1:5.json");
        (bytes memory pv, bytes memory proof) = abi.decode(
            vm.parseJson(fixture),
            (bytes, bytes)
        );

        // First 32 bytes of the public values of the pv are the trusted block hash.
        // Read 32 bytes from pv.
        bytes32 trustedBlockHash = bytes32(this.readBytes(pv, 0, 32));

        // Initialize SP1Tendermint with program hash, verifier and the trusted block hash.
        SP1Tendermint sp1 = new SP1Tendermint(
            programHash,
            address(verifier),
            trustedBlockHash
        );

        // Update SP1Tendermint using the proof from the fixture.
        sp1.updateHeader(pv, proof);
    }
}
