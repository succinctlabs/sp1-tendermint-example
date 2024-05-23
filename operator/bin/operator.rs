use alloy_sol_types::{sol, SolCall};
use log::info;
use sp1_sdk::utils::setup_logger;
use std::time::Duration;
use tendermint_operator::{contract::ContractClient, TendermintProver};

sol! {
    contract SP1Tendermint {
        bytes32 public latestHeader;

        function verifyTendermintProof(
            bytes calldata proof,
            bytes calldata publicValues
        ) public;
    }
}

/// An implementation of a Tendermint Light Client operator that will poll the latest block from
/// an onchain Tendermint light client. Then it will generate a proof of the latest block periodically
/// and update the light client contract with the proof.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    setup_logger();

    // Instantiate a contract client to interact with the deployed Solidity Tendermint contract.
    let contract_client = ContractClient::default();

    // Instantiate a Tendermint prover based on the environment variable.
    let prover = TendermintProver::new();

    loop {
        // Read the existing trusted header hash from the contract.
        let latest_header_call_data = SP1Tendermint::latestHeaderCall {}.abi_encode();
        let trusted_header_hash = contract_client.read(latest_header_call_data).await?;

        // Generate a header update proof from the trusted block to the latest block.
        let trusted_block_height = prover
            .get_block_height_from_hash(&trusted_header_hash)
            .await;
        let latest_block_height = prover.fetch_latest_block_height().await;
        let (trusted_light_block, target_light_block) = prover
            .fetch_light_blocks(trusted_block_height, latest_block_height)
            .await;
        let proof_data =
            prover.generate_tendermint_proof(&trusted_light_block, &target_light_block);

        // Relay the proof to the contract.
        let proof_as_bytes = proof_data.proof.encoded_proof.into_bytes();
        info!("Proof bytes: {:?}", hex::encode(proof_as_bytes));
        let verify_tendermint_proof_call_data = SP1Tendermint::verifyTendermintProofCall {
            publicValues: proof_data.public_values.to_vec().into(),
            proof: proof_as_bytes.into(),
        }
        .abi_encode();

        contract_client
            .send(verify_tendermint_proof_call_data)
            .await?;

        // Sleep for 60 seconds.
        println!("sleeping for 60 seconds");
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
