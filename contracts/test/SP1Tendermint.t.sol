// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1Tendermint} from "../src/SP1Tendermint.sol";
import {SP1Verifier} from "@sp1-contracts/v1.1.0/SP1Verifier.sol";
import {SP1MockVerifier} from "@sp1-contracts/SP1MockVerifier.sol";

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
    SP1Tendermint public mockTendermint;

    function setUp() public {
        SP1TendermintFixtureJson memory fixture = loadFixture("fixture.json");
        SP1Verifier verifier = new SP1Verifier();
        tendermint = new SP1Tendermint(
            fixture.vkey,
            fixture.trustedHeaderHash,
            fixture.trustedHeight,
            address(verifier)
        );

        SP1TendermintFixtureJson memory mockFixture = loadFixture(
            "mock_fixture.json"
        );
        SP1MockVerifier mockVerifier = new SP1MockVerifier();
        mockTendermint = new SP1Tendermint(
            mockFixture.vkey,
            mockFixture.trustedHeaderHash,
            mockFixture.trustedHeight,
            address(mockVerifier)
        );
    }

    function loadFixture(
        string memory fileName
    ) public view returns (SP1TendermintFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/fixtures/", fileName);
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
        SP1TendermintFixtureJson memory fixture = loadFixture("fixture.json");

        tendermint.verifyTendermintProof(fixture.proof, fixture.publicValues);

        assert(tendermint.latestHeader() == fixture.targetHeaderHash);
        assert(tendermint.latestHeight() == fixture.targetHeight);
    }

    // Confirm that submitting an empty proof fails.
    function testRevert_InvalidTendermint() public {
        SP1TendermintFixtureJson memory fixture = loadFixture("fixture.json");

        // Create a fake proof.
        bytes memory fakeProof = new bytes(fixture.proof.length);

        // Create fixture of the length of the proof bytes.
        tendermint.verifyTendermintProof(fakeProof, fixture.publicValues);
    }

    // Confirm that submitting an empty proof passes the mock verifier.
    function test_ValidMockTendermint() public {
        SP1TendermintFixtureJson memory fixture = loadFixture(
            "mock_fixture.json"
        );

        mockTendermint.verifyTendermintProof(bytes(""), fixture.publicValues);

        assert(mockTendermint.latestHeader() == fixture.targetHeaderHash);
        assert(mockTendermint.latestHeight() == fixture.targetHeight);
    }

    // Confirm that submitting a non-empty proof with the mock verifier fails. This typically
    // indicates that the user has passed in a real proof to the mock verifier.
    function testRevert_Invalid_MockTendermint() public {
        SP1TendermintFixtureJson memory fixture = loadFixture(
            "mock_fixture.json"
        );

        mockTendermint.verifyTendermintProof(bytes("aa"), fixture.publicValues);

        assert(mockTendermint.latestHeader() == fixture.targetHeaderHash);
        assert(mockTendermint.latestHeight() == fixture.targetHeight);
    }
}
