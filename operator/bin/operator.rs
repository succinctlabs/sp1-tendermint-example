use alloy::{sol, sol_types::SolCall};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::time::Duration;
use tendermint_operator::{client::ContractClient, fetch_inputs};

sol! {
    contract SP1Tendermint {
        bytes32 public latestHeader;

        function updateHeader(
            bytes calldata publicValues,
            bytes calldata proof
        ) public;
    }
}

const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let prover_client = ProverClient::new();
    let contract_client = ContractClient::default();

    loop {
        // Read the existing trusted header hash from the contract.
        let latest_header_call_data = SP1Tendermint::latestHeaderCall {}.abi_encode();
        let trusted_header_hash = contract_client.read(latest_header_call_data).await?;

        let (trusted_light_block, target_light_block) = fetch_inputs(&trusted_header_hash).await?;

        // Some types don't serialize with bincode (native serializer of sp1-sdk), LightBlock is an example.
        // TODO: LIGHT_BLOCK SHOULD SERIALIZE IN THE VM NATIVELY
        let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
        let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

        let mut stdin = SP1Stdin::new();
        stdin.write_vec(encoded_1);
        stdin.write_vec(encoded_2);

        // Generate a header update proof to the latest block.
        // Note: We currently use prove_remote_async, as prove has an async block_on,
        // that prevents it from being used in an async thread.
        let mut proof = prover_client
            .prove_remote_async(TENDERMINT_ELF, stdin)
            .await
            .expect("proving failed");

        // Relay the proof to the contract.
        let update_header_call_data = SP1Tendermint::updateHeaderCall {
            publicValues: proof.public_values.read(),
            proof: vec![].into(),
        }
        .abi_encode();

        contract_client.send(update_header_call_data).await?;

        // Sleep for 10 seconds.
        println!("sleeping for 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
