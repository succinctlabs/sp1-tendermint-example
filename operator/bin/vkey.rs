use sp1_sdk::types::MockProver;
use tendermint_operator::TENDERMINT_ELF;

/// Generates the vkey digest for the tendermint program.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    sp1_sdk::utils::setup_logger();

    let prover = MockProver::new();
    let key = prover.get_vk_digest(TENDERMINT_ELF);
    println!("vkey digest: {:?}", hex::encode(key));

    Ok(())
}
