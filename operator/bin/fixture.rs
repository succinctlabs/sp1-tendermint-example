use clap::Parser;
use std::fs;
use tendermint_operator::generate_header_update_proof_between_blocks;

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
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let args = FixtureArgs::parse();

    // Generate a header update proof for the specified blocks.
    let proof_data =
        generate_header_update_proof_between_blocks(args.trusted_block, args.target_block).await;

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
