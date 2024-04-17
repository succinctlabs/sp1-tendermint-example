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

    // Note: Uses PRIVATE_KEY by default.
    let client = ProverClient::new();

    // Submit the proof request to the prover network and poll for the proof.
    let proof = client
        .prove_remote_async(TENDERMINT_ELF, stdin)
        .await
        .expect("proving failed");

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

    // BLOCK_INTERVAL defines which block to update to next.
    let block_interval: u64 = 10;

    // Read environment variables.
    let rpc_url = env::var("RPC_URL").unwrap();
    let private_key = env::var("PRIVATE_KEY").unwrap();
    let contract_address = env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS not set");
    let contract_address: Address =
        Address::from_str(&contract_address).expect("CONTRACT_ADDRESS not valid");

    loop {
        let provider =
            Provider::<Http>::try_from(rpc_url.clone()).expect("could not connect to client");

        let signer = LocalWallet::from_str(&private_key)
            .unwrap()
            .with_chain_id(provider.get_chainid().await.unwrap().as_u64());

        let tendermint_client = TendermintRPCClient::default();
        let client = Arc::new(SignerMiddleware::new(provider.clone(), signer.clone()));

        let contract = SP1Tendermint::new(contract_address.0 .0, client);

        let trusted_header_hash = contract.latest_header().await?;

        println!("Trusted header hash: {:?}", trusted_header_hash);

        let trusted_light_block = tendermint_client
            .get_light_block_by_hash(&trusted_header_hash)
            .await;
        let trusted_block_height = trusted_light_block.signed_header.header.height.value();
        println!(
            "Trusted light block height: {}",
            trusted_light_block.signed_header.header.height.value()
        );

        // Find next block.
        let next_block_height =
            trusted_block_height + block_interval - (trusted_block_height % block_interval);

        // Get latest block.
        let latest_block_height = tendermint_client.get_latest_block_height().await;

        if next_block_height < latest_block_height {
            let (trusted_light_block, target_light_block) = tendermint_client
                .get_light_blocks(&trusted_header_hash, next_block_height)
                .await;

            // Generate a proof of the update from trusted_light_block to target_light_block and get the corresponding
            // proof data.
            let proof_data =
                generate_header_update_proof(&trusted_light_block, &target_light_block).await;

            // Relay the proof to the contract.
            // TODO: Parse errors nicely.
            let tx: Option<TransactionReceipt> = contract
                .update(proof_data.pv.into(), proof_data.proof.into())
                .send()
                .await?
                .await?;
            if tx.is_some() {
                println!("Transaction hash: {:?}", tx.unwrap().transaction_hash);
            } else {
                println!("Transaction failed");
            }

            println!(
                "successfully generated and verified proof for the program! relayed to contract"
            );
        }
        // Sleep for 10 seconds.
        println!("sleeping for 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
