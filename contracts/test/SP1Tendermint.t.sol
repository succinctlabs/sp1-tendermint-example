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
    function readBytes(bytes calldata bs, uint256 start, uint256 end) public pure returns (bytes memory) {
        return bs[start:end];
    }

    function deserializeProof(bytes memory proofBytes)
        public
        pure
        returns (
            uint256[8] memory proof,
            uint32 commitmentCount,
            uint256[2] memory commitments,
            uint256[2] memory commitmentPok
        )
    {
        // Ensure the proofBytes has the exact necessary length.
        require(proofBytes.length == 8 * 32 + 4 + 2 * 32 + 2 * 32, "Invalid data length");

        uint256 offset = 32;

        // Deserialize the proof.
        for (uint256 i = 0; i < 8; i++) {
            assembly {
                mstore(add(proof, add(0, mul(32, i))), mload(add(proofBytes, add(offset, mul(32, i)))))
            }
        }

        // Deserialize commitment count.
        offset += 8 * 32;
        assembly {
            let dataLocation := add(proofBytes, offset)
            let loadedData := mload(dataLocation)
            commitmentCount := and(shr(224, loadedData), 0xFFFFFFFF)
        }

        offset += 4;
        for (uint256 i = 0; i < 2; i++) {
            assembly {
                mstore(add(commitments, add(0, mul(32, i))), mload(add(proofBytes, add(offset, mul(32, i)))))
            }
        }

        offset += 2 * 32;
        for (uint256 i = 0; i < 2; i++) {
            assembly {
                mstore(add(commitmentPok, add(0, mul(32, i))), mload(add(proofBytes, add(offset, mul(32, i)))))
            }
        }
    }

    function test_decodeProof() public {
        // bytes memory proofBytes =
        //     hex"0003a4d46bfb32d27d0479439997fac6bca2f437d1eab416a9bc27649789ced82d457800b3f8b4cfeb69675650df56cbc5250a4dac0fb65fd7d0adda1619c02a19a6d84be26aa2195a797bcdf61cc6932eab7385b2eba46cbfd6d1e6f367a192131e5a5b7eb5962916fdd06069ebfe1be67ef25a7ecc6c42f3a4f0edb0ca54d026e62fe3cb65665c70134bd5a164f26f69da792ec08b577506d05c35990c5d8a18e115b8f8f6234b2052b678b3359e09d3057bdb1c70fe61b90708005406306b00f9391ec607770c7b1a42b72a20025a4ddfa952762be145136b009e8238394e2b24e9d53fa3dcab07466c4aa78e61ff90fd179edae4afe7ed6a9095a7eef52d000000010fb2fa4abd67c7936aa77110d81f5c566a0b94e17710dd1c8af00b3b02f985a9263ef747d817f467232cab6c8f724934f266b4d7e2f4526c852b6870e020d3d02b2856e607394faffc588e9d0a7452d0c9371907d2a782e36be0e0cdf4527fd114342828ab77b21559334edc212c3407e88eb13ddc9c27688ff17cae2c9c0f54";
        bytes memory proofBytes = hex"2ed683356fc0036f0773cd27d055c1cf355e3a468fc4234cfd46508361dfeeaa2d2718fc3e58b5d1297ddd72b7f6d89715aeaf3012d01ebd072324344d623b4923667caf024df9551fedc9b3bd3311c4228af4d65a1bce215ccc10e5326c14890eb24bdd198439f9ac4e0fbdb3b96f07079ae61d0dc27b19bfc48604393d2456184f6bf9c0502bcf5ddbe0f2a2dbb20ff1ca6f02ec83cbb29ef9af5a9c67a6e50dd1b1742af0a9a9076650b473d6cf1cbf4413aaec09e0248b4c5f0574f94c5028bb6fd0affed355d58dc4f36b829b933adc6e21aa458fadb09b63f8c550af7f1f58dd4463df0f66c4d210614dc2893b72c4e687039b7b808050d163ae8c060a0000000110eb729644ff519065bcdc70b4231cec6537daad95e7cc609f29a88515d5a43e12cbe75ddd776f76f3d9b1be5caaef62087b098e7a06efd4894622b1c86fd05e159d8f6ec2a3ff119975f2cc1b6426cafca00442f07365c09f44d5d903296b8a0d2e3992b4447fdda797cd884622c47ce228ebfb279f5da708520f4d74ae1463";
        (
            uint256[8] memory proof,
            uint32 commitmentCount,
            uint256[2] memory commitments,
            uint256[2] memory commitmentPok
        ) = deserializeProof(proofBytes);
        for (uint256 i = 0; i < 8; i++) {
            console.log("proof[%d]: %d", i, proof[i]);
        }
        console.log("commitmentCount: %d", commitmentCount);
        for (uint256 i = 0; i < 2; i++) {
            console.log("commitments[%d]: %d", i, commitments[i]);
        }
        for (uint256 i = 0; i < 2; i++) {
            console.log("commitmentPok[%d]: %d", i, commitmentPok[i]);
        }
    }

    // Read from fixture.
    function test_fixtureTest() public {
        // Parse JSON {pv, proof} from fixture as bytes.
        string memory fixture = vm.readFile("./fixtures/fixture_2:6.json");
        (bytes memory pv, bytes memory proof_) = abi.decode(vm.parseJson(fixture), (bytes, bytes));


        bytes memory proof = hex"256aa08fd561465035fab1da608647d4b414771f4c4366fd6b149176c936beae15a4ee84c0e9261e2aaf515396e9a2ad0ca8ba2e70bfb90af2be45c7d663135c09e902a849f8cb619af318ac1725007749b261a04dfd4ea2839c8d601631f306006173266ae91276fdfbbb24ec77adf1bcb22d3ba9bbade883725b9232f8f30e26ed475aeefdb490ed29315509b2bb26f5e7ef30c57a0b98f5a40600359195840b42549b8cc558457c93d3262d4f8623f82c0c1e8d2b0974a8bd57490a6d9ac60b1af15433a483b48f9db928dd54cd6ff4cbe3da5eacece6a56b8a05386557f11ed654a22a6baf9dea6cf6e6bbf1f10879c03f4c1a1638093de839015b591a330000000110eb729644ff519065bcdc70b4231cec6537daad95e7cc609f29a88515d5a43e12cbe75ddd776f76f3d9b1be5caaef62087b098e7a06efd4894622b1c86fd05e159d8f6ec2a3ff119975f2cc1b6426cafca00442f07365c09f44d5d903296b8a0d2e3992b4447fdda797cd884622c47ce228ebfb279f5da708520f4d74ae1463";
 
        // First 32 bytes of the public values of the pv are the trusted block hash.
        // Read 32 bytes from pv.
        bytes32 trustedBlockHash = bytes32(this.readBytes(pv, 0, 32));

        // Initialize SP1Tendermint with program hash, verifier and the trusted block hash.
        SP1Tendermint sp1 = new SP1Tendermint(programHash, address(verifier), trustedBlockHash);

        // Update SP1Tendermint using the proof from the fixture.
        sp1.updateHeader(pv, proof);
    }
}
