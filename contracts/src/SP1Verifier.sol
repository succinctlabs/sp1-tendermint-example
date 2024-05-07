// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Verifier} from "./Groth16Verifier.sol";

contract SP1Verifier is Verifier {
    function deserializeProof(bytes memory proofBytes) public pure returns (
        uint256[8] memory proof,
        uint32 commitmentCount,
        uint256[2] memory commitments,
        uint256[2] memory commitmentPok
    ) {
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

    function verifySP1Proof(
        bytes32 vkeyHash,
        bytes memory proofBytes,
        bytes memory publicValues
    ) public view {
        (
            uint256[8] memory proof,
            uint32 commitmentCount,
            uint256[2] memory commitments,
            uint256[2] memory commitmentPok
        ) = deserializeProof(proofBytes);

        uint256[2] memory publicInputs;
        publicInputs[0] = 159097377863346869977797037726919643140737446196733537169767291768498911756;
        publicInputs[1] = 13745541382856617652488105939243667921607621289075412439081280875484670377749;
        this.verifyProof(proof, commitments, commitmentPok, publicInputs);
    }
}
