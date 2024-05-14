use sp1_prover::HashableKey;
use sp1_sdk::{artifacts::export_solidity_groth16_verifier, MockProver, Prover};
use std::path::PathBuf;
use tendermint_operator::TENDERMINT_ELF;

/// Exports the Solidity verifier and generates the vkey digest for the tendermint program.
/// Note that this must be run after running the "fixture" script, as that will build the groth16
/// artifacts and store them in the ~/.sp1 directory. If those artifacts are not present, this
/// script will fail.
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
