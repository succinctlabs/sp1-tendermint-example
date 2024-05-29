use sp1_sdk::{ProverClient, SP1ProvingKey, SP1Stdin, SP1VerifyingKey};
use tendermint_light_client_verifier::types::LightBlock;

pub mod contract;
mod types;
pub mod util;

// The path to the ELF file for the Succinct zkVM program.
pub const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

pub struct TendermintProver {
    pub prover_client: ProverClient,
    pub pkey: SP1ProvingKey,
    pub vkey: SP1VerifyingKey,
}

impl Default for TendermintProver {
    fn default() -> Self {
        Self::new()
    }
}

impl TendermintProver {
    pub fn new() -> Self {
        log::info!("Initializing SP1 ProverClient...");
        let prover_client = ProverClient::new();
        let (pkey, vkey) = prover_client.setup(TENDERMINT_ELF);
        log::info!("SP1 ProverClient initialized");
        Self {
            prover_client,
            pkey,
            vkey,
        }
    }

    /// Generate a proof of an update from trusted_light_block to target_light_block. Returns an
    /// SP1Groth16Proof.
    pub fn generate_tendermint_proof(
        &self,
        trusted_light_block: &LightBlock,
        target_light_block: &LightBlock,
    ) -> SP1Groth16Proof {
        // Encode the light blocks to be input into our program.
        let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
        let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

        // Write the encoded light blocks to stdin.
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(encoded_1);
        stdin.write_vec(encoded_2);

        // Generate the proof. Depending on SP1_PROVER env variable, this may be a mock, local or network proof.
        let proof = self
            .prover_client
            .prove_plonk(&self.pkey, stdin)
            .expect("proving failed");

        // Verify proof.
        self.prover_client
            .verify_plonk(&proof, &self.vkey)
            .expect("Verification failed");

        // Return the proof.
        proof
    }
}
