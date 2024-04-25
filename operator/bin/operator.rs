use alloy::{sol, sol_types::SolCall};
use sp1_sdk::{types::MockProver, ProverClient};
use std::{env, time::Duration};
use tendermint_operator::{
    client::ContractClient, MockTendermintProver, RealTendermintProver, TendermintProver,
};

sol! {
    contract SP1Tendermint {
        bytes32 public latestHeader;

        function updateHeader(
            bytes calldata publicValues,
            bytes calldata proof
        ) public;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let contract_client = ContractClient::default();
    let real_proofs = env::var("REAL_PROOFS").unwrap_or("false".to_string()) == "true";
    let prover: Box<dyn TendermintProver> = if real_proofs {
        let prover_client = ProverClient::new();
        Box::new(RealTendermintProver::new(prover_client))
    } else {
        Box::new(MockTendermintProver::new(MockProver::new()))
    };

    loop {
        // Read the existing trusted header hash from the contract.
        let latest_header_call_data = SP1Tendermint::latestHeaderCall {}.abi_encode();
        let trusted_header_hash = contract_client.read(latest_header_call_data).await?;

        // Generate a header update proof to the latest block.
        let proof_data = prover
            .generate_header_update_proof_to_latest_block(&trusted_header_hash)
            .await;

        // Relay the proof to the contract.
        if let Ok(proof_data) = proof_data {
            let update_header_call_data = SP1Tendermint::updateHeaderCall {
                publicValues: proof_data.pv.into(),
                proof: proof_data.proof.into(),
            }
            .abi_encode();

            contract_client.send(update_header_call_data).await?;
        }

        // Sleep for 10 seconds.
        println!("sleeping for 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
