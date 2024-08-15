#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::{sol, SolValue};
use core::time::Duration;
use tendermint_light_client_verifier::{
    options::Options, types::LightBlock, ProdVerifier, Verdict, Verifier,
};

sol! {
    struct TendermintOutput {
        uint64 trustedHeight;
        uint64 targetHeight;
        bytes32 trustedHeaderHash;
        bytes32 targetHeaderHash;
    }
}

fn main() {
    // Read in 2 encoded vectors of two light blocks from the zkVM's stdin.
    let encoded_1 = sp1_zkvm::io::read_vec();
    let encoded_2 = sp1_zkvm::io::read_vec();

    // Decode the light blocks.
    let light_block_1: LightBlock = serde_cbor::from_slice(&encoded_1).unwrap();
    let light_block_2: LightBlock = serde_cbor::from_slice(&encoded_2).unwrap();

    let vp = ProdVerifier::default();
    let opt = Options {
        trust_threshold: Default::default(),
        // 2 week trusting period.
        trusting_period: Duration::from_secs(14 * 24 * 60 * 60),
        clock_drift: Default::default(),
    };
    let verify_time = light_block_2.time() + Duration::from_secs(20);
    let verdict = vp.verify_update_header(
        light_block_2.as_untrusted_state(),
        light_block_1.as_trusted_state(),
        &opt,
        verify_time.unwrap(),
    );

    match verdict {
        Verdict::Success => {
            println!(
                "Verified light client update from height {} to height {}!",
                light_block_1.signed_header.header.height.value(),
                light_block_2.signed_header.header.height.value()
            );
        }
        v => panic!("Failed to verify light client update: {:?}", v),
    }

    // Now that we have verified our proof, we commit the header hashes to the zkVM to expose
    // them as public values.
    let header_hash_1 = light_block_1.signed_header.header.hash();
    let header_hash_1: [u8; 32] = header_hash_1.as_bytes().to_vec().try_into().unwrap();
    let header_hash_2 = light_block_2.signed_header.header.hash();
    let header_hash_2: [u8; 32] = header_hash_2.as_bytes().to_vec().try_into().unwrap();

    let output = TendermintOutput {
        trustedHeight: light_block_1.signed_header.header.height.value(),
        targetHeight: light_block_2.signed_header.header.height.value(),
        trustedHeaderHash: header_hash_1.into(),
        targetHeaderHash: header_hash_2.into(),
    };

    sp1_zkvm::io::commit_slice(&output.abi_encode());
}
