use clap::Parser;
use tendermint_operator::TendermintProver;

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
/// RUST_LOG=info cargo run --bin fixture --release -- --trusted-block=1 --target-block=5
/// ```
/// The fixture will be written to the path: ./contracts/fixtures/fixture_1:5.json
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let args = FixtureArgs::parse();

    // std::env::set_var("SP1_PROVER", "mock");
    let prover = TendermintProver::new();

    // Generate a header update proof for the specified blocks.
    let proof_data = prover
        .generate_header_update_proof_between_blocks(args.trusted_block, args.target_block)
        .await;

    // Write the proof data to the file path.
    let file_path = format!(
        "{}/fixture_{}:{}.json",
        args.fixture_path, args.trusted_block, args.target_block
    );
    proof_data.save(file_path)?;

    Ok(())
}
