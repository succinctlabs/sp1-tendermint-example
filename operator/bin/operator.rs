use alloy::{sol, sol_types::SolCall};
use std::time::Duration;
use tendermint_operator::{client::ContractClient, TendermintProver};

sol! {
    contract SP1Tendermint {
        bytes32 public latestHeader;

        function updateHeader(
            bytes calldata publicValues,
            bytes calldata proof
        ) public;
    }
}

/// An implementation of a Tendermint Light Client operator that will poll the latest block from
/// an onchain Tendermint light client. Then it will generate a proof of the latest block periodically
/// and update the light client contract with the proof.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // Instantiate a contract client to interact with the deployed Solidity Tendermint contract.
    let contract_client = ContractClient::default();

    // Instantiate a Tendermint prover based on the environment variable.
    let prover = TendermintProver::new();

    loop {
        // Read the existing trusted header hash from the contract.
        let latest_header_call_data = SP1Tendermint::latestHeaderCall {}.abi_encode();
        let trusted_header_hash = contract_client.read(latest_header_call_data).await?;

        // Generate a header update proof to the latest block.
        let proof_data = prover
            .generate_header_update_proof_to_latest_block(&trusted_header_hash)
            .await;

        if let Err(e) = proof_data {
            log::error!("Failed to generate proof: {:?}", e);
            continue;
        }

        // Relay the proof to the contract.
        if let Ok(proof_data) = proof_data {
            let proof_as_bytes = proof_data.proof.encoded_proof.into_bytes();
            let update_header_call_data = SP1Tendermint::updateHeaderCall {
                publicValues: proof_data.public_values.to_vec().into(),
                proof: proof_as_bytes.into(),
            }
            .abi_encode();

            contract_client.send(update_header_call_data).await?;
        }

        // Sleep for 10 seconds.
        println!("sleeping for 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
