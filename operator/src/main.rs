use std::env;
use std::str::FromStr;
use std::time::Duration;

use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;
use reqwest::Client;
use sp1_sdk::{utils, ProverClient, PublicValues, SP1Stdin};

use sha2::{Digest, Sha256};
use subtle_encoding::hex;
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::types::LightBlock;
use tendermint_light_client_verifier::ProdVerifier;
use tendermint_light_client_verifier::Verdict;
use tendermint_light_client_verifier::Verifier;

use crate::util::fetch_block;
use crate::util::fetch_commit;
use crate::util::fetch_latest_commit;
use crate::util::fetch_light_block;

use alloy::sol;

sol! {
    #[sol(rpc)]
    contract SP1Tendermint {
        #[derive(Debug)]
        bytes32 public latestHeader;

        function update(
            bytes calldata _publicValues,
            bytes calldata _proof
        );
    }
}

const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
mod util;

async fn get_latest_block_height() -> u64 {
    let url = "https://celestia-mocha-rpc.publicnode.com:443/commit";
    let client = Client::new();
    let latest_commit = fetch_latest_commit(&client, url).await.unwrap();
    latest_commit.result.signed_header.header.height.value()
}

async fn get_light_block_by_hash(hash: &[u8]) -> LightBlock {
    let peer_id: [u8; 20] = [
        0x72, 0x6b, 0xc8, 0xd2, 0x60, 0x38, 0x7c, 0xf5, 0x6e, 0xcf, 0xad, 0x3a, 0x6b, 0xf6, 0xfe,
        0xcd, 0x90, 0x3e, 0x18, 0xa2,
    ];
    const BASE_URL: &str = "https://celestia-mocha-rpc.publicnode.com:443";

    let url = format!(
        "{}/block_by_hash?hash=0x{}",
        BASE_URL,
        String::from_utf8(hex::encode(hash)).unwrap()
    );
    let client = Client::new();
    let block = fetch_block(&client, &url).await.unwrap();
    fetch_light_block(block.result.block.header.height.value(), peer_id, BASE_URL)
        .await
        .unwrap()
}

async fn get_light_blocks(
    trusted_header_hash: &[u8],
    target_block_height: u64,
) -> (LightBlock, LightBlock) {
    // Uniquely identify a peer in the network.
    let peer_id: [u8; 20] = [
        0x72, 0x6b, 0xc8, 0xd2, 0x60, 0x38, 0x7c, 0xf5, 0x6e, 0xcf, 0xad, 0x3a, 0x6b, 0xf6, 0xfe,
        0xcd, 0x90, 0x3e, 0x18, 0xa2,
    ];
    const BASE_URL: &str = "https://celestia-mocha-rpc.publicnode.com:443";

    let block_by_hash_url = format!(
        "{}/block_by_hash?hash={}",
        BASE_URL,
        String::from_utf8(hex::encode(trusted_header_hash)).unwrap()
    );

    let client = Client::new();

    let trusted_block = fetch_block(&client, &block_by_hash_url).await.unwrap();
    let trusted_height = trusted_block.result.block.header.height.value();

    let trusted_light_block = fetch_light_block(trusted_height, peer_id, BASE_URL)
        .await
        .expect("Failed to generate light block 1");
    let target_light_block = fetch_light_block(target_block_height, peer_id, BASE_URL)
        .await
        .expect("Failed to generate light block 2");
    (trusted_light_block, target_light_block)
}

// Return the public values and proof.
async fn prove_next_block_height_update(
    trusted_light_block: &LightBlock,
    target_light_block: &LightBlock,
) -> (Vec<u8>, Vec<u8>) {
    let expected_verdict = verify_blocks(trusted_light_block, target_light_block);

    let mut stdin = SP1Stdin::new();

    let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
    let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

    stdin.write_vec(encoded_1);
    stdin.write_vec(encoded_2);

    // TODO: normally we could just write the LightBlock, but bincode doesn't work with LightBlock.
    // The following code will panic.
    // let encoded: Vec<u8> = bincode::serialize(&light_block_1).unwrap();
    // let decoded: LightBlock = bincode::deserialize(&encoded[..]).unwrap();

    let client = ProverClient::new();

    let proof = client
        .prove_remote_async(TENDERMINT_ELF, stdin)
        .await
        .expect("proving failed");

    // Verify proof.
    client
        .verify(TENDERMINT_ELF, &proof)
        .expect("verification failed");

    // Verify the public values
    let mut pv_hasher = Sha256::new();
    pv_hasher.update(trusted_light_block.signed_header.header.hash().as_bytes());
    pv_hasher.update(target_light_block.signed_header.header.hash().as_bytes());
    pv_hasher.update(&serde_cbor::to_vec(&expected_verdict).unwrap());
    let expected_pv_digest: &[u8] = &pv_hasher.finalize();

    let public_values_bytes = proof.proof.shard_proofs[0].public_values.clone();
    let public_values = PublicValues::from_vec(public_values_bytes);
    assert_eq!(
        public_values.commit_digest_bytes().as_slice(),
        expected_pv_digest
    );

    // Return the public values.
    // TODO: Until Groth16 wrapping is implemented, return empty bytes for the proof.
    (public_values.commit_digest_bytes(), vec![])
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // Generate proof.
    utils::setup_logger();

    // BLOCK_INTERVAL defines which block to update to next.
    let block_interval: u64 = 1000;

    // Read private key from environment variable.
    let private_key = env::var("PRIVATE_KEY").unwrap();

    // Read RPC URL from environment variable.
    let rpc_url = env::var("RPC_URL").unwrap();

    loop {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_builtin(&rpc_url)
            .await
            .expect("could not connect to client");

        let address: Address = Address::from_str(&env::var("CONTRACT_ADDRESS").unwrap()).unwrap();

        let contract = SP1Tendermint::new(address, provider);

        let latest_header_hash_call = contract.latestHeader();

        let trusted_header_hash = latest_header_hash_call.call().await.unwrap();

        println!("Trusted header hash: {:?}", trusted_header_hash);

        let trusted_light_block = get_light_block_by_hash(&trusted_header_hash._0.0).await;
        let trusted_block_height = trusted_light_block.signed_header.header.height.value();
        println!(
            "Trusted light block height: {}",
            trusted_light_block.signed_header.header.height.value()
        );

        // Find next block.
        let next_block_height =
            trusted_block_height + block_interval - (trusted_block_height % block_interval);

        // Get latest block.
        let latest_block_height = get_latest_block_height().await;

        if next_block_height < latest_block_height {
            let (trusted_light_block, target_light_block) =
                get_light_blocks(&trusted_header_hash._0.0, next_block_height).await;

            // Discard the proof bytes for now and update the
            let (pv, proof) =
                prove_next_block_height_update(&trusted_light_block, &target_light_block).await;

            // Relay the proof to the contract.
            let update_call = contract.update(pv.into(), proof.into());
            let _ = update_call.call().await.unwrap();

            println!(
                "successfully generated and verified proof for the program! relayed to contract"
            );
        }
        // Sleep for 10 seconds.
        println!("sleeping for 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

fn verify_blocks(trusted_light_block: &LightBlock, target_light_block: &LightBlock) -> Verdict {
    let vp = ProdVerifier::default();
    let opt = Options {
        trust_threshold: Default::default(),
        // 2 week trusting period.
        trusting_period: Duration::from_secs(14 * 24 * 60 * 60),
        clock_drift: Default::default(),
    };
    // TODO: Change this to the actual time.
    // For now, this works as we can test as if the current time is right after the target block.
    let verify_time = target_light_block.time() + Duration::from_secs(20);
    vp.verify_update_header(
        target_light_block.as_untrusted_state(),
        trusted_light_block.as_trusted_state(),
        &opt,
        verify_time.unwrap(),
    )
}
