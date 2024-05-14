use clap::Parser;
use log::info;
use std::env;
use tendermint_operator::TendermintProver;
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
/// RUST_LOG=info cargo run --bin test --release -- --trusted-block=<TRUSTED_BLOCK> --target-block=<TARGET_BLOCK>
/// ```
// TODO: When https://github.com/succinctlabs/sp1/pull/687 is merged, we can make this an async
// program as block_in_place will handle prove_groth16.
fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let args = ScriptArgs::parse();

    // Set the prover to mock mode. This is useful for testing the program.
    env::set_var("SP1_PROVER", "mock");
    let prover = TendermintProver::new();

    let rt = runtime::Runtime::new()?;

    // Fetch the inputs for the proof.
    let (trusted_light_block, target_light_block) = rt.block_on(async {
        prover
            .fetch_light_blocks(args.trusted_block, args.target_block)
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
