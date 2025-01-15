use clap::Parser;
use sp1_sdk::{utils::setup_logger, CpuProver, HashableKey, Prover};
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

    // Generate the vkey hash to use in the contract.
    let prover = CpuProver::mock();
    let (_, vk) = prover.setup(TENDERMINT_ELF);
    let tendermint_client = TendermintRPCClient::default();

    let (trusted_height, trusted_header_hash) = if let Some(trusted_block) = args.trusted_block {
        let commit = tendermint_client.get_commit(trusted_block).await?;
        (trusted_block, commit.result.signed_header.header.hash())
    } else {
        let latest_commit = tendermint_client.get_latest_commit().await?;
        (
            latest_commit.result.signed_header.header.height.value(),
            latest_commit.result.signed_header.header.hash(),
        )
    };

    println!(
        "TENDERMINT_VKEY_HASH={} TRUSTED_HEIGHT={} TRUSTED_HEADER_HASH={}",
        vk.bytes32(),
        trusted_height,
        trusted_header_hash
    );

    Ok(())
}
