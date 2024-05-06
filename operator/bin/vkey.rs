use sp1_prover::HashableKey;
use sp1_sdk::{
    artifacts::{export_solidity_verifier, WrapCircuitType},
    MockProver, Prover,
};
use std::path::PathBuf;
use tendermint_operator::{words_to_bytes_be, TENDERMINT_ELF};

/// Exports the Solidity verifier and generates the vkey digest for the tendermint program.
/// Note that this must be run after running the "fixture" script, as that will build the groth16
/// artifacts and store them in the ~/.sp1 directory. If those artifacts are not present, this
/// script will fail.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    // Export the solidity verifier to the contracts/src directory.
    export_solidity_verifier(
        WrapCircuitType::Groth16,
        PathBuf::from("../contracts/src"),
        None,
    )
    .expect("failed to export verifier");

    // Now generate the vkey digest to use in the contract.
    let prover = MockProver::new();
    let (_, vk) = prover.setup(TENDERMINT_ELF);
    let digest = words_to_bytes_be(&vk.hash_u32());
    print!("VKEY_DIGEST={}", hex::encode(digest));

    Ok(())
}
