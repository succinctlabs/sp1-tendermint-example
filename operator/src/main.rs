use crate::util::TendermintRPCClient;
use alloy::primitives::Address;
use ethers::{
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::TransactionReceipt,
};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::{env, str::FromStr, sync::Arc, time::Duration};
use tendermint_light_client_verifier::types::LightBlock;

abigen!(SP1Tendermint, "../abi/SP1Tendermint.abi.json");

const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
mod util;

struct ProofData {
    pv: Vec<u8>,
    proof: Vec<u8>,
}

// Generate a proof of an update from trusted_light_block to target_light_block. Returns the public values and proof
// of the update.
async fn generate_header_update_proof(
    trusted_light_block: &LightBlock,
    target_light_block: &LightBlock,
) -> ProofData {
    let mut stdin = SP1Stdin::new();

    // TODO: normally we could just write the LightBlock, but bincode doesn't work with LightBlock.
    // let encoded: Vec<u8> = bincode::serialize(&light_block_1).unwrap();
    // let decoded: LightBlock = bincode::deserialize(&encoded[..]).unwrap();
    let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
    let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

    stdin.write_vec(encoded_1);
    stdin.write_vec(encoded_2);

    // Note: Uses PRIVATE_KEY by default to initialize the client.
    let client = ProverClient::new();

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // Read environment variables.
    let rpc_url = env::var("RPC_URL").unwrap();
    let private_key = env::var("PRIVATE_KEY").unwrap();
    let contract_address = env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS not set");
    let contract_address: Address =
        Address::from_str(&contract_address).expect("CONTRACT_ADDRESS not valid");

    // Initialize client for interacting with the Tendermint chain.
    let tendermint_client = TendermintRPCClient::default();

    loop {
        // Initialize the client for interacting with the SP1Tendermint contract.
        let provider =
            Provider::<Http>::try_from(rpc_url.clone()).expect("could not connect to client");
        let signer = LocalWallet::from_str(&private_key)
            .unwrap()
            .with_chain_id(provider.get_chainid().await.unwrap().as_u64());
        let client = Arc::new(SignerMiddleware::new(provider.clone(), signer.clone()));
        let contract = SP1Tendermint::new(contract_address.0 .0, client);

        // Read the existing trusted header hash from the contract.
        let trusted_header_hash = contract.latest_header().await?;
        println!("Trusted header hash: {:?}", trusted_header_hash);

        // Get the block height corresponding to the trusted header hash.
        let trusted_block_height = tendermint_client
            .get_block_height_from_hash(&trusted_header_hash)
            .await;
        println!("Trusted light block height: {}", trusted_block_height);

        // Get the latest block from the Tendermint chain.
        let latest_block_height = tendermint_client.get_latest_block_height().await;

        // Generate a proof of the update from trusted_block_height to latest_block_height and get the corresponding
        // proof data.
        let (trusted_light_block, target_light_block) = tendermint_client
            .get_light_blocks(trusted_block_height, latest_block_height)
            .await;
        let proof_data =
            generate_header_update_proof(&trusted_light_block, &target_light_block).await;

        // Relay the proof to the contract.
        let tx: Option<TransactionReceipt> = contract
            .update(proof_data.pv.into(), proof_data.proof.into())
            .send()
            .await?
            .await?;

        if let Some(tx) = tx {
            println!(
                "Successfully relayed proof! Transaction hash: {:?}",
                tx.transaction_hash
            );
        }

        // Sleep for 10 seconds.
        println!("sleeping for 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
