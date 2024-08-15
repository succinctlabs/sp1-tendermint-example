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

/// An implementation of a Tendermint Light Client operator that will poll an onchain Tendermint
/// light client and generate a proof of the transition from the latest block in the contract to the
/// latest block on the chain. Then, submits the proof to the contract and updates the contract with
/// the latest block hash and height.
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
        let contract_latest_height = SP1Tendermint::latestHeightCall {}.abi_encode();
        let contract_latest_height = contract_client.read(contract_latest_height).await?;
        let contract_latest_height = U256::abi_decode(&contract_latest_height, true).unwrap();
        let trusted_block_height: u64 = contract_latest_height.try_into().unwrap();

        if trusted_block_height == 0 {
            panic!(
                "No trusted height found on the contract. Something is wrong with the contract."
            );
        }

        let chain_latest_block_height = tendermint_rpc_client.get_latest_block_height().await;
        let (trusted_light_block, target_light_block) = tendermint_rpc_client
            .get_light_blocks(trusted_block_height, chain_latest_block_height)
            .await;

        // Generate a proof of the transition from the trusted block to the target block.
        let proof_data =
            prover.generate_tendermint_proof(&trusted_light_block, &target_light_block);

        // Construct the on-chain call and relay the proof to the contract.
        let verify_tendermint_proof_call_data = SP1Tendermint::verifyTendermintProofCall {
            publicValues: proof_data.public_values.to_vec().into(),
            proof: proof_data.bytes().into(),
        }
        .abi_encode();
        contract_client
            .send(verify_tendermint_proof_call_data)
            .await?;

        info!(
            "Updated the latest block of Tendermint light client at address {} from block {} to block {}.",
            contract_client.contract, trusted_block_height, chain_latest_block_height
        );

        // Sleep for 60 seconds.
        debug!("sleeping for 60 seconds");
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
