use sp1_prover::HashableKey;
use sp1_sdk::{MockProver, Prover};
use std::path::PathBuf;
use tendermint_operator::{words_to_bytes_be, TENDERMINT_ELF};

/// Exports the Solidity verifier and generates the vkey digest for the tendermint program.
///
/// Note that this must be run after running the "fixture" script, as that will build the groth16
/// artifacts and store them in the ~/.sp1 directory. If those artifacts are not present, this
/// script will fail.
fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    tracing::info!("building groth16 artifacts");
    let artifacts_dir = sp1_prover::artifacts::get_groth16_artifacts_dir();
    sp1_prover::artifacts::build_groth16_artifacts(artifacts_dir);

    tracing::info!("exporting groth16 verifier");
    let contracts_src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src");
    sp1_sdk::artifacts::export_solidity_groth16_verifier(contracts_src_dir)
        .expect("failed to export verifier");

    tracing::info!("exporting vkey digest");
    let prover = MockProver::new();
    let (_, vk) = prover.setup(TENDERMINT_ELF);
    let digest = words_to_bytes_be(&vk.hash_u32());
    print!("VKEY_DIGEST={}", hex::encode(digest));
    Ok(())
}
