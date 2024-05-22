use clap::Parser;
use tendermint_operator::util::TendermintRPCClient;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct GenesisArgs {
    /// Trusted block.
    #[clap(long)]
    trusted_block: Option<u64>,
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

    let args = GenesisArgs::parse();

    let tendermint_client = TendermintRPCClient::default();

    if let Some(trusted_block) = args.trusted_block {
        let commit = tendermint_client.fetch_commit(trusted_block).await?;
        print!(
            "TRUSTED_HEADER_HASH={}",
            commit.result.signed_header.header.hash()
        );
    } else {
        let latest_commit = tendermint_client.fetch_latest_commit().await?;
        print!(
            "TRUSTED_HEADER_HASH={}",
            latest_commit.result.signed_header.header.hash()
        );
    }

    Ok(())
}
