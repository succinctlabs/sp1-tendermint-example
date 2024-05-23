use clap::Parser;
use log::info;
use sp1_sdk::utils::setup_logger;
use tendermint_operator::{util::TendermintRPCClient, TendermintProver};
use tokio::runtime;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ScriptArgs {
    /// Trusted block.
    #[clap(long)]
    trusted_block: u64,

    /// Target block.
    #[clap(long, env)]
    target_block: u64,
}

/// Generate a proof between the given trusted and target blocks.
/// Example:
/// ```
/// RUST_LOG=info SP1_PROVER=mock cargo run --bin test --release -- --trusted-block=<TRUSTED_BLOCK> --target-block=<TARGET_BLOCK>
/// ```
fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    setup_logger();

    let args = ScriptArgs::parse();

    let tendermint_rpc_client = TendermintRPCClient::default();
    let prover = TendermintProver::new();

    let rt = runtime::Runtime::new()?;

    // Fetch the inputs for the proof.
    let (trusted_light_block, target_light_block) = rt.block_on(async {
        tendermint_rpc_client
            .get_light_blocks(args.trusted_block, args.target_block)
            .await
    });

    // Generate the proof.
    let proof = prover.generate_tendermint_proof(&trusted_light_block, &target_light_block);

    // Verify proof.
    prover
        .prover_client
        .verify_groth16(&proof, &prover.vkey)
        .expect("Verification failed");

    info!("Successfully generated proof!");

    Ok(())
}
