// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";
import {SP1Verifier} from "../src/SP1Verifier.sol";

struct SP1TendermintFixtureJson {
    bytes32 trustedHeaderHash;
    bytes32 targetHeaderHash;
    bytes32 vkey;
    bytes publicValues;
    bytes proof;
}

contract SP1TendermintTest is Test {
    using stdJson for string;

    SP1Tendermint public tendermint;

    function setUp() public {
        SP1TendermintFixtureJson memory fixture = loadFixture();
        console.log("Challenge: %s", string(abi.encodePacked(fixture.vkey)));
        console.log(
            "Trusted Header Hash: %s",
            string(abi.encodePacked(fixture.trustedHeaderHash))
        );
        tendermint = new SP1Tendermint(fixture.vkey, fixture.trustedHeaderHash);
    }

    function loadFixture()
        public
        view
        returns (SP1TendermintFixtureJson memory)
    {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/fixtures/fixture.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (SP1TendermintFixtureJson));
    }

    function test_ValidTendermint() public {
        SP1TendermintFixtureJson memory fixture = loadFixture();

        tendermint.verifyTendermintProof(fixture.proof, fixture.publicValues);

        assert(tendermint.latestHeader() == fixture.targetHeaderHash);
    }

    function testFail_InvalidTendermint() public {
        SP1TendermintFixtureJson memory fixture = loadFixture();

        tendermint.verifyTendermintProof(
            fixture.publicValues,
            fixture.publicValues
        );

        assert(tendermint.latestHeader() == fixture.targetHeaderHash);
    }
}
