use alloy_primitives::U256;
use alloy_sol_types::{sol, SolCall, SolValue};
use log::{debug, info};
use sp1_sdk::utils::setup_logger;
use std::time::Duration;
use tendermint_operator::{contract::ContractClient, util::TendermintRPCClient, TendermintProver};

sol! {
    contract SP1Tendermint {
        bytes32 public latestHeader;
        uint64 public latestHeight;

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
    let tendermint_rpc_client = TendermintRPCClient::default();
    let prover = TendermintProver::new();

    loop {
        // Read the existing trusted header hash from the contract.
        let latest_height_call_data = SP1Tendermint::latestHeightCall {}.abi_encode();
        let latest_height = contract_client.read(latest_height_call_data).await?;
        let latest_height = U256::abi_decode(&latest_height, true).unwrap();
        let trusted_block_height: u64 = latest_height.try_into().unwrap();

        // let latest_height: u64 = latest_height.try_into().unwrap();
        if trusted_block_height == 0 {
            panic!("No trusted height found in the contract, likely using an outdated contract.");
        }

        let latest_block_height = tendermint_rpc_client.get_latest_block_height().await;
        let (trusted_light_block, target_light_block) = tendermint_rpc_client
            .get_light_blocks(trusted_block_height, latest_block_height)
            .await;
        let proof_data =
            prover.generate_tendermint_proof(&trusted_light_block, &target_light_block);

        // Relay the proof to the contract.
        let proof_as_bytes = hex::decode(&proof_data.proof.encoded_proof).unwrap();
        let verify_tendermint_proof_call_data = SP1Tendermint::verifyTendermintProofCall {
            publicValues: proof_data.public_values.to_vec().into(),
            proof: proof_as_bytes.into(),
        }
        .abi_encode();

        contract_client
            .send(verify_tendermint_proof_call_data)
            .await?;

        info!(
            "Updated contract's latest block from {} to {}.",
            trusted_block_height, latest_block_height
        );

        // Sleep for 60 seconds.
        debug!("sleeping for 60 seconds");
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
