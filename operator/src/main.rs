use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use alloy::primitives::Address;
use ethers::contract::abigen;
use ethers::middleware::SignerMiddleware;
use ethers::providers::Middleware;
use ethers::signers::Signer;
use ethers::types::TransactionReceipt;
use sp1_sdk::{utils, ProverClient, SP1Stdin};

use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;

use tendermint_light_client_verifier::types::LightBlock;

use crate::util::get_latest_block_height;
use crate::util::get_light_block_by_hash;
use crate::util::get_light_blocks;

abigen!(SP1Tendermint, "../abi/SP1Tendermint.abi.json");

const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
mod util;

struct ProofData {
    pv: Vec<u8>,
    proof: Vec<u8>,
}

// Return the public values and proof.
async fn prove_next_block_height_update(
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

    // Read SP1_PRIVATE_KEY from environment variable.
    let sp1_private_key = env::var("SP1_PRIVATE_KEY").unwrap();
    let client = ProverClient::new().with_network(sp1_private_key.as_str());

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
async fn main() {
    dotenv::dotenv().ok();
    // Generate proof.
    utils::setup_logger();

    // BLOCK_INTERVAL defines which block to update to next.
    let block_interval: u64 = 10;

    // Read private key from environment variable.
    let private_key = env::var("PRIVATE_KEY").unwrap();

    // Read RPC URL from environment variable.
    let rpc_url = env::var("RPC_URL").unwrap();

    loop {
        let provider =
            Provider::<Http>::try_from(rpc_url.clone()).expect("could not connect to client");

        let signer = LocalWallet::from_str(&private_key)
            .unwrap()
            .with_chain_id(provider.get_chainid().await.unwrap().as_u64());

        let client = Arc::new(SignerMiddleware::new(provider.clone(), signer.clone()));

        let address: Address = Address::from_str(&env::var("CONTRACT_ADDRESS").unwrap()).unwrap();

        let contract = SP1Tendermint::new(address.0 .0, client);

        let trusted_header_hash = contract.latest_header().await.unwrap();

        println!("Trusted header hash: {:?}", trusted_header_hash);

        let trusted_light_block = get_light_block_by_hash(&trusted_header_hash).await;
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
                get_light_blocks(&trusted_header_hash, next_block_height).await;

            // Discard the proof bytes for now and update the
            let proof_data =
                prove_next_block_height_update(&trusted_light_block, &target_light_block).await;

            // Relay the proof to the contract.
            // TODO: Parse errors nicely.
            let tx: Option<TransactionReceipt> = contract
                .update(proof_data.pv.into(), proof_data.proof.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap();
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
