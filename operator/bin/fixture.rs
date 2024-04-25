use clap::Parser;
use sp1_sdk::ProverClient;
use std::{env, fs};
use tendermint_operator::{MockTendermintProver, RealTendermintProver, TendermintProver};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct FixtureArgs {
    /// Trusted block.
    #[clap(long)]
    trusted_block: u64,

    /// Target block.
    #[clap(long, env)]
    target_block: u64,

    /// Fixture path.
    #[clap(long, default_value = "../contracts/fixtures")]
    fixture_path: String,
}

/// Writes the proof data for the given trusted and target blocks to the given fixture path.
/// Example:
/// ```
/// cargo run --bin fixture -- trusted_block=1 target_block=5
/// ```
/// The fixture will be written to the path: ./contracts/fixtures/fixture_1:5.json
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let args = FixtureArgs::parse();
    let prover_client = ProverClient::new();

    // Generate a header update proof for the specified blocks.
    let proof_data = if env::var("REAL_PROOFS").unwrap_or("false".to_string()) == "true" {
        RealTendermintProver::generate_header_update_proof_between_blocks(
            &prover_client,
            args.trusted_block,
            args.target_block,
        )
        .await
    } else {
        MockTendermintProver::generate_header_update_proof_between_blocks(
            &prover_client,
            args.trusted_block,
            args.target_block,
        )
        .await
    };

    if let Ok(proof_data) = proof_data {
        // Write the proof data to JSON.
        let proof_data_json = serde_json::to_string(&proof_data)?;

        let file_path = format!(
            "{}/fixture_{}:{}.json",
            args.fixture_path, args.trusted_block, args.target_block
        );

        // Write the proof data to the fixture path.
        fs::write(file_path, proof_data_json)?;
    }

    Ok(())
}
