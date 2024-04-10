use anyhow::Result;
use std::env;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sp1_sdk::client::{NetworkClient, StarkGenericConfig};
use sp1_sdk::proto::network::ProofStatus;
use sp1_sdk::utils::BabyBearPoseidon2;
use sp1_sdk::{utils, SP1ProofWithIO, SP1Prover, SP1Stdin, SP1Verifier};

use sha2::{Digest, Sha256};
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::types::LightBlock;
use tendermint_light_client_verifier::ProdVerifier;
use tendermint_light_client_verifier::Verdict;
use tendermint_light_client_verifier::Verifier;
use tokio::time::sleep;

use log::info;

use crate::util::fetch_latest_commit;
use crate::util::fetch_light_block;

const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
mod util;

async fn poll_proof<SC: for<'de> Deserialize<'de> + Serialize + StarkGenericConfig>(
    network_client: NetworkClient,
    proof_id: &str,
) -> Result<SP1ProofWithIO<SC>> {
    // Query every 10 seconds for the proof status.
    // TODO: Proof status should be an object (instead of a tuple).
    // TODO: THe STARK config is annoying.

    const POLL_INTERVAL: u64 = 10;
    const MAX_NUM_POLLS: u64 = 1000;

    for _ in 0..MAX_NUM_POLLS {
        info!("Polling proof status");
        let proof_status = network_client.get_proof_status::<SC>(proof_id).await;
        if let Ok(proof_status) = proof_status {
            info!("Proof status: {:?}", proof_status.0.status());
            if proof_status.0.status() == ProofStatus::ProofSucceeded {
                if let Some(proof_data) = proof_status.1 {
                    return Ok(proof_data);
                }
            }
        }
        sleep(Duration::from_secs(POLL_INTERVAL)).await;
    }

    Err(anyhow::anyhow!("Proof failed or was rejected"))
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // Generate proof.
    utils::setup_logger();

    let network_client = NetworkClient::with_token(env::var("SP1_NETWORK_TOKEN").expect("SP1_NETWORK_TOKEN not set"));

    // // Uniquely identify a peer in the network.
    // let peer_id: [u8; 20] = [
    //     0x72, 0x6b, 0xc8, 0xd2, 0x60, 0x38, 0x7c, 0xf5, 0x6e, 0xcf, 0xad, 0x3a, 0x6b, 0xf6, 0xfe,
    //     0xcd, 0x90, 0x3e, 0x18, 0xa2,
    // ];
    // const BASE_URL: &str = "https://celestia-mocha-rpc.publicnode.com:443";
    // let client = Client::new();
    // let url = format!("{}/commit", BASE_URL);
    // let latest_commit = fetch_latest_commit(&client, &url).await.unwrap();
    // let block: u64 = latest_commit.result.signed_header.header.height.into();
    // println!("Latest block: {}", block);

    // let light_block_1 = fetch_light_block(block - 20, peer_id, BASE_URL)
    //     .await
    //     .expect("Failed to generate light block 1");

    // let light_block_2 = fetch_light_block(block, peer_id, BASE_URL)
    //     .await
    //     .expect("Failed to generate light block 2");

    // let expected_verdict = verify_blocks(light_block_1.clone(), light_block_2.clone());

    // let mut input = serde_cbor::to_vec(&light_block_1).unwrap();
    // input.extend(serde_cbor::to_vec(&light_block_2).unwrap());

    // // TODO: normally we could just write the LightBlock, but bincode doesn't work with LightBlock.
    // // The following code will panic.
    // // let encoded: Vec<u8> = bincode::serialize(&light_block_1).unwrap();
    // // let decoded: LightBlock = bincode::deserialize(&encoded[..]).unwrap();

    // let proof_id = network_client.create_proof(TENDERMINT_ELF, &input).await;

    // info!("Proof ID: {:?}", proof_id);

    let proof_id: Result<&str> = Ok("proofrequest_01hv2rfkcyfgd8t1smpkcd31tv");

    if let Ok(proof_id) = proof_id {
        type SC = BabyBearPoseidon2;
        let proof = poll_proof::<SC>(network_client, &proof_id).await;
        if let Ok(valid_proof) = proof {
            info!("Proof: {:?}", valid_proof.public_values.buffer.data);
        }
    }
}

fn verify_blocks(light_block_1: LightBlock, light_block_2: LightBlock) -> Verdict {
    let vp = ProdVerifier::default();
    let opt = Options {
        trust_threshold: Default::default(),
        trusting_period: Duration::from_secs(500),
        clock_drift: Default::default(),
    };
    let verify_time = light_block_2.time() + Duration::from_secs(20);
    vp.verify_update_header(
        light_block_2.as_untrusted_state(),
        light_block_1.as_trusted_state(),
        &opt,
        verify_time.unwrap(),
    )
}
