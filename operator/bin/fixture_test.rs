use alloy_sol_types::{sol, SolType};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, ProverClient};
use std::path::PathBuf;
use tendermint_operator::TendermintProver;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct FixtureArgs {
    /// Fixture path.
    #[clap(long, default_value = "../contracts/fixtures")]
    fixture_path: String,
}

type TendermintProofTuple = sol! {
    tuple(bytes32, bytes32)
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TendermintFixture {
    trusted_header_hash: String,
    target_header_hash: String,
    vkey: String,
    public_values: String,
    proof: String,
}

/// Writes the proof data for the given trusted and target blocks to the given fixture path.
/// Example:
/// ```
/// RUST_LOG=info cargo run --bin fixture --release -- --trusted-block=1 --target-block=5
/// ```
/// The fixture will be written to the path: ./contracts/fixtures/fixture_1:5.json
fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let args = FixtureArgs::parse();

    let prover = TendermintProver::new();

    // Save the proof data to the file path.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(args.fixture_path);

    let contents = std::fs::read_to_string(fixture_path.join("fixture.json")).unwrap();
    let fixture: TendermintFixture = serde_json::from_str(&contents).unwrap();

    // let prover_client = ProverClient::new();
    // prover_client
    //     .verify_groth16(&fixture.proof, &fixture.vkey)
    //     .expect("Verification failed");

    Ok(())
}
