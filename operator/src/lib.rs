use crate::util::TendermintRPCClient;
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1Stdin};
use tendermint_light_client_verifier::types::LightBlock;

pub mod client;
pub mod util;

const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Serialize, Deserialize)]
pub struct ProofData {
    pub pv: Vec<u8>,
    pub proof: Vec<u8>,
}

// Generate a proof using the trusted_header_hash, fetch the latest block and generate a proof for that.
pub async fn generate_header_update_proof_to_latest_block(
    trusted_header_hash: &[u8],
) -> anyhow::Result<ProofData> {
    let tendermint_client = TendermintRPCClient::default();
    let latest_block_height = tendermint_client.get_latest_block_height().await;
    // Get the block height corresponding to the trusted header hash.
    let trusted_block_height = tendermint_client
        .get_block_height_from_hash(trusted_header_hash)
        .await;
    println!(
        "SP1Tendermint contract's latest block height: {}",
        trusted_block_height
    );
    let (trusted_light_block, target_light_block) = tendermint_client
        .get_light_blocks(trusted_block_height, latest_block_height)
        .await;
    let proof_data = generate_header_update_proof(&trusted_light_block, &target_light_block).await;
    Ok(proof_data)
}

pub async fn generate_header_update_proof_between_blocks(
    trusted_block_height: u64,
    target_block_height: u64,
) -> anyhow::Result<ProofData> {
    let tendermint_client = TendermintRPCClient::default();
    let (trusted_light_block, target_light_block) = tendermint_client
        .get_light_blocks(trusted_block_height, target_block_height)
        .await;
    let proof_data = generate_header_update_proof(&trusted_light_block, &target_light_block).await;
    Ok(proof_data)
}

// Generate a proof of an update from trusted_light_block to target_light_block. Returns the public values and proof
// of the update.
pub async fn generate_header_update_proof(
    trusted_light_block: &LightBlock,
    target_light_block: &LightBlock,
) -> ProofData {
    // Note: Uses PRIVATE_KEY by default to initialize the client.
    let client = ProverClient::new();

    // TODO: normally we could just write the LightBlock, but bincode doesn't work with LightBlock.
    // let encoded: Vec<u8> = bincode::serialize(&light_block_1).unwrap();
    // let decoded: LightBlock = bincode::deserialize(&encoded[..]).unwrap();
    let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
    let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

    let mut stdin = SP1Stdin::new();
    stdin.write_vec(encoded_1);
    stdin.write_vec(encoded_2);

    // Submit the proof request to the prover network and poll for the proof.
    let proof = client
        .prove_remote_async(TENDERMINT_ELF, stdin)
        .await
        .expect("proving failed");
    println!("Successfully generated proof!");

    // Verify proof.
    client
        .verify(TENDERMINT_ELF, &proof)
        .expect("verification failed");
    println!("Successfully verified proof!");

    // Return the public values.
    // TODO: Until Groth16 wrapping is implemented, return empty bytes for the proof.
    ProofData {
        pv: proof.public_values.buffer.data,
        proof: vec![],
    }
}
