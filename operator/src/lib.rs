use crate::util::TendermintRPCClient;
use sp1_sdk::{
    types::{Prover, SP1PlonkProof},
    ProverClient, SP1Prover, SP1Stdin,
};
use tendermint_light_client_verifier::types::LightBlock;

pub mod client;
pub mod util;

const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

// Generate a proof using the trusted_header_hash, fetch the latest block and generate a proof for that.
pub async fn generate_header_update_proof_to_latest_block(
    prover_client: &ProverClient,
    trusted_header_hash: &[u8],
) -> anyhow::Result<SP1PlonkProof> {
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
    println!("got height");
    let proof_data =
        generate_header_update_proof(&prover_client, &trusted_light_block, &target_light_block)
            .await;
    println!("got proof");
    Ok(proof_data)
}

pub async fn generate_header_update_proof_between_blocks(
    prover_client: &ProverClient,
    trusted_block_height: u64,
    target_block_height: u64,
) -> anyhow::Result<SP1PlonkProof> {
    let tendermint_client = TendermintRPCClient::default();
    let (trusted_light_block, target_light_block) = tendermint_client
        .get_light_blocks(trusted_block_height, target_block_height)
        .await;
    let proof_data =
        generate_header_update_proof(prover_client, &trusted_light_block, &target_light_block)
            .await;
    Ok(proof_data)
}

// Generate a proof of an update from trusted_light_block to target_light_block. Returns the public values and proof
// of the update.
pub async fn generate_header_update_proof(
    prover_client: &ProverClient,
    trusted_light_block: &LightBlock,
    target_light_block: &LightBlock,
) -> SP1PlonkProof {
    // TODO: normally we could just write the LightBlock, but bincode doesn't work with LightBlock.
    // let encoded: Vec<u8> = bincode::serialize(&light_block_1).unwrap();
    // let decoded: LightBlock = bincode::deserialize(&encoded[..]).unwrap();
    let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
    let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

    let mut stdin = SP1Stdin::new();
    stdin.write_vec(encoded_1);
    stdin.write_vec(encoded_2);

    // Generate the proof.
    println!("proving");
    let proof = prover_client
        .prove_plonk(TENDERMINT_ELF, stdin)
        .expect("proving failed");
    println!("Successfully generated proof!");

    // Verify proof.
    // client
    //     .verify(TENDERMINT_ELF, &proof)
    //     .expect("verification failed");
    // println!("Successfully verified proof!");

    proof
}
