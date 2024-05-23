use clap::Parser;
use sp1_sdk::{utils::setup_logger, HashableKey, MockProver, Prover};
use tendermint_operator::{util::TendermintRPCClient, TENDERMINT_ELF};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct GenesisArgs {
    /// Trusted block.
    #[clap(long)]
    trusted_block: Option<u64>,
}

/// Fetches the trusted header hash for the given block height. Defaults to the latest block height.
/// Example:
/// ```
/// RUST_LOG=info cargo run --bin genesis --release
/// ```
///
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    setup_logger();

    let args = GenesisArgs::parse();

    // Generate the vkey digest to use in the contract.
    let prover = MockProver::new();
    let (_, vk) = prover.setup(TENDERMINT_ELF);
    println!("VKEY_DIGEST={}", vk.bytes32());

    let tendermint_client = TendermintRPCClient::default();

    if let Some(trusted_block) = args.trusted_block {
        let commit = tendermint_client.get_commit(trusted_block).await?;
        println!("TRUSTED_HEIGHT={}", trusted_block);
        println!(
            "TRUSTED_HEADER_HASH={}",
            commit.result.signed_header.header.hash()
        );
    } else {
        let latest_commit = tendermint_client.get_latest_commit().await?;
        println!(
            "TRUSTED_HEIGHT={}",
            latest_commit.result.signed_header.header.height.value()
        );
        println!(
            "TRUSTED_HEADER_HASH={}",
            latest_commit.result.signed_header.header.hash()
        );
    }

    Ok(())
}
