use sp1_sdk::{artifacts::export_solidity_groth16_verifier, HashableKey, MockProver, Prover};
use std::path::PathBuf;
use tendermint_operator::TENDERMINT_ELF;

/// Exports the Solidity verifier and generates the vkey digest for the tendermint program.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    // Export the solidity verifier to the contracts/src directory.
    export_solidity_groth16_verifier(PathBuf::from("../contracts/src"))
        .expect("failed to export verifier");

    // Now generate the vkey digest to use in the contract.
    let prover = MockProver::new();
    let (_, vk) = prover.setup(TENDERMINT_ELF);
    print!("VKEY_DIGEST={}", vk.bytes32());

    Ok(())
}
