use alloy_sol_types::{sol, SolType};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sp1_sdk::{utils::setup_logger, HashableKey};
use std::path::PathBuf;
use tendermint_operator::{util::TendermintRPCClient, TendermintProver};

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
    setup_logger();

    let args = FixtureArgs::parse();

    let tendermint_rpc_client = TendermintRPCClient::default();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (trusted_light_block, target_light_block) = runtime.block_on(async {
        tendermint_rpc_client
            .get_light_blocks(args.trusted_block, args.target_block)
            .await
    });

    let prover = TendermintProver::new();
    // Generate a header update proof for the specified blocks.
    let proof_data = prover.generate_tendermint_proof(&trusted_light_block, &target_light_block);

    let bytes = proof_data.public_values.as_slice();
    let (trusted_header_hash, target_header_hash) =
        TendermintProofTuple::abi_decode(bytes, false).unwrap();

    let fixture = TendermintFixture {
        trusted_header_hash: hex::encode(trusted_header_hash),
        target_header_hash: hex::encode(target_header_hash),
        vkey: prover.vkey.bytes32(),
        public_values: proof_data.public_values.bytes(),
        proof: proof_data.bytes(),
    };

    // Save the proof data to the file path.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(args.fixture_path);
    std::fs::write(
        fixture_path.join("fixture.json"),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .unwrap();

    Ok(())
}
