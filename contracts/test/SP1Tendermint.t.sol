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
    uint64 trustedHeight;
    uint64 targetHeight;
    bytes32 vkey;
    bytes publicValues;
    bytes proof;
}

contract SP1TendermintTest is Test {
    using stdJson for string;

    SP1Tendermint public tendermint;

    function setUp() public {
        SP1TendermintFixtureJson memory fixture = loadFixture();
        tendermint = new SP1Tendermint(
            fixture.vkey,
            fixture.trustedHeaderHash,
            fixture.trustedHeight
        );
    }

    function loadFixture()
        public
        view
        returns (SP1TendermintFixtureJson memory)
    {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/fixtures/fixture.json");
        string memory json = vm.readFile(path);
        bytes32 trustedHeaderHash = json.readBytes32(".trustedHeaderHash");
        bytes32 targetHeaderHash = json.readBytes32(".targetHeaderHash");
        uint64 trustedHeight = uint64(json.readUint(".trustedHeight"));
        uint64 targetHeight = uint64(json.readUint(".targetHeight"));
        bytes32 vkey = json.readBytes32(".vkey");
        bytes memory publicValues = json.readBytes(".publicValues");
        bytes memory proof = json.readBytes(".proof");

        SP1TendermintFixtureJson memory fixture = SP1TendermintFixtureJson({
            trustedHeaderHash: trustedHeaderHash,
            targetHeaderHash: targetHeaderHash,
            trustedHeight: trustedHeight,
            targetHeight: targetHeight,
            vkey: vkey,
            publicValues: publicValues,
            proof: proof
        });

        return fixture;
    }

    function test_ValidTendermint() public {
        SP1TendermintFixtureJson memory fixture = loadFixture();

        tendermint.verifyTendermintProof(fixture.proof, fixture.publicValues);

        assert(tendermint.latestHeader() == fixture.targetHeaderHash);
        assert(tendermint.latestHeight() == fixture.targetHeight);
    }

    function testFail_InvalidTendermint() public {
        SP1TendermintFixtureJson memory fixture = loadFixture();

        // Create a fake proof.
        bytes memory fakeProof = new bytes(fixture.proof.length);

        // Create fixture of the length of the proof bytes.
        tendermint.verifyTendermintProof(fakeProof, fixture.publicValues);
    }
}
