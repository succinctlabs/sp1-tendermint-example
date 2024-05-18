// // SPDX-License-Identifier: UNLICENSED
// pragma solidity ^0.8.13;

// import {Script, console} from "forge-std/Script.sol";
// import {stdJson} from "forge-std/StdJson.sol";
// import {SP1Verifier} from "../src/SP1Verifier.sol";
// import {SP1Tendermint} from "../src/SP1Tendermint.sol";

// struct SP1TendermintFixtureJson {
//     bytes32 trustedHeaderHash;
//     bytes32 targetHeaderHash;
//     bytes32 vkey;
//     bytes publicValues;
//     bytes proof;
// }

// contract SP1TendermintScript is Script {
//     function setUp() public {}

//     function loadFixture()
//         public
//         view
//         returns (SP1TendermintFixtureJson memory)
//     {
//         string memory root = vm.projectRoot();
//         string memory path = string.concat(root, "/fixtures/fixture.json");
//         string memory json = vm.readFile(path);
//         bytes memory jsonBytes = json.parseRaw(".");
//         return abi.decode(jsonBytes, (SP1TendermintFixtureJson));
//     }

//     function run() public returns (address) {
//         vm.startBroadcast();

//         SP1TendermintFixtureJson memory fixture = loadFixture();

//         // Deploy SP1Tendermint.
//         SP1Tendermint sp1 = new SP1Tendermint(
//             fixture.vkey,
//             fixture.trustedHeaderHash
//         );
//         return address(sp1);
//     }
// }
