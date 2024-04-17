#![no_main]
sp1_zkvm::entrypoint!(main);

use core::time::Duration;
use tendermint_light_client_verifier::{
    options::Options, types::LightBlock, ProdVerifier, Verdict, Verifier,
};

fn main() {
    // Normally we could just do this to read in the LightBlocks, but bincode doesn't work with LightBlock.
    // This is likely a bug in tendermint-rs.
    // let light_block_1 = sp1_zkvm::io::read::<LightBlock>();
    // let light_block_2 = sp1_zkvm::io::read::<LightBlock>();

    let encoded_1 = sp1_zkvm::io::read_vec();
    let encoded_2 = sp1_zkvm::io::read_vec();

    let light_block_1: LightBlock = serde_cbor::from_slice(&encoded_1).unwrap();
    let light_block_2: LightBlock = serde_cbor::from_slice(&encoded_2).unwrap();

    let header_hash_1 = light_block_1.signed_header.header.hash();
    let header_hash_2 = light_block_2.signed_header.header.hash();

    sp1_zkvm::io::commit_slice(header_hash_1.as_bytes());
    sp1_zkvm::io::commit_slice(header_hash_2.as_bytes());

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

    let verdict_encoded = serde_cbor::to_vec(&verdict).unwrap();
    sp1_zkvm::io::commit_slice(verdict_encoded.as_slice());

    match verdict {
        Verdict::Success => {
            println!("success");
        }
        v => panic!("expected success, got: {:?}", v),
    }
}
