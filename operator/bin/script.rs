use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use tendermint_operator::{util::TendermintRPCClient, TENDERMINT_ELF};
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
/// RUST_LOG=info cargo run --bin script --release -- --trusted-block=1 --target-block=5
/// ```
fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let args = ScriptArgs::parse();

    let prover_client = ProverClient::new();
    let (pkey, vkey) = prover_client.setup(TENDERMINT_ELF);

    let rt = runtime::Runtime::new()?;

    // Fetch the input light blocks.
    let (trusted_light_block, target_light_block) = rt.block_on(async {
        let tendermint_client = TendermintRPCClient::default();
        tendermint_client
            .get_light_blocks(args.trusted_block, args.target_block)
            .await
    });

    // Encode the light blocks to be input into our program.
    let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
    let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

    // Write the encoded light blocks to stdin.
    let mut stdin = SP1Stdin::new();
    stdin.write_vec(encoded_1);
    stdin.write_vec(encoded_2);

    // Generate the proof. Depending on SP1_PROVER env, this may be a local or network proof.
    let proof = prover_client
        .prove_groth16(&pkey, stdin)
        .expect("proving failed");
    println!("Successfully generated proof!");

    // Verify proof.
    prover_client
        .verify_groth16(&proof, &vkey)
        .expect("Verification failed");

    Ok(())
}
