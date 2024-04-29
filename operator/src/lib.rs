// TODO: import from sp1_sdk when these are public in the future
use crate::util::TendermintRPCClient;
use alloy::{primitives::Uint, sol, sol_types::SolValue};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sp1_sdk::{prove::MockProver, ProverClient, SP1ProvingKey, SP1Stdin, SP1VerifyingKey};
use std::str::FromStr;
use tendermint_light_client_verifier::types::LightBlock;

pub mod client;
pub mod util;

// The path to the ELF file for the Succinct zkVM program.
pub const TENDERMINT_ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

/// Proof data ready to be sent to the contract.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProofData {
    pub pv: Vec<u8>,
    pub proof: Vec<u8>,
}

// The Groth16 proof ABI.
sol! {
    struct Groth16Proof {
        uint256[2] a;
        uint256[2][2] b;
        uint256[2] c;
    }
}

#[async_trait]
pub trait TendermintProver: Send + Sync {
    /// Using the trusted_header_hash, fetch the latest block and generate a proof for that.
    async fn generate_header_update_proof_to_latest_block(
        &self,
        trusted_header_hash: &[u8],
    ) -> anyhow::Result<ProofData> {
        let tendermint_client = TendermintRPCClient::default();
        let latest_block_height = tendermint_client.get_latest_block_height().await;
        // Get the block height corresponding to the trusted header hash.
        let trusted_block_height = tendermint_client
            .get_block_height_from_hash(trusted_header_hash)
            .await;
        log::info!(
            "SP1Tendermint contract's latest block height: {}",
            trusted_block_height
        );
        log::info!(
            "Generating proof for blocks {} to {} (latest)",
            trusted_block_height,
            latest_block_height
        );
        let (trusted_light_block, target_light_block) = tendermint_client
            .get_light_blocks(trusted_block_height, latest_block_height)
            .await;
        let proof_data = self
            .generate_header_update_proof(&trusted_light_block, &target_light_block)
            .await;
        Ok(proof_data)
    }

    async fn generate_header_update_proof_between_blocks(
        &self,
        trusted_block_height: u64,
        target_block_height: u64,
    ) -> anyhow::Result<ProofData> {
        log::info!(
            "Generating proof for blocks {} to {}",
            trusted_block_height,
            target_block_height
        );
        let tendermint_client = TendermintRPCClient::default();
        let (trusted_light_block, target_light_block) = tendermint_client
            .get_light_blocks(trusted_block_height, target_block_height)
            .await;
        let proof_data = self
            .generate_header_update_proof(&trusted_light_block, &target_light_block)
            .await;
        Ok(proof_data)
    }

    /// Generate a proof of an update from trusted_light_block to target_light_block. Returns the
    /// public values and proof of the update.
    async fn generate_header_update_proof(
        &self,
        trusted_light_block: &LightBlock,
        target_light_block: &LightBlock,
    ) -> ProofData {
        // Encode the light blocks to be input into our program.
        let encoded_1 = serde_cbor::to_vec(&trusted_light_block).unwrap();
        let encoded_2 = serde_cbor::to_vec(&target_light_block).unwrap();

        // Write the encoded light blocks to stdin.
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(encoded_1);
        stdin.write_vec(encoded_2);

        // Generate proof.
        self.generate_proof(stdin).await
    }

    // This function is used to generate a proof. Implementations of this trait can implement
    // this method with either a local or network prover.
    async fn generate_proof(&self, stdin: SP1Stdin) -> ProofData;
}

pub struct RealTendermintProver {
    prover_client: ProverClient,
    pkey: SP1ProvingKey,
    vkey: SP1VerifyingKey,
}

impl RealTendermintProver {
    pub fn new(prover_client: ProverClient) -> Self {
        // With the prover_client, set up the proving key and verifying key.
        let (pkey, vkey) = prover_client.setup(TENDERMINT_ELF);
        Self {
            prover_client,
            pkey,
            vkey,
        }
    }
}

#[async_trait]
impl TendermintProver for RealTendermintProver {
    async fn generate_proof(&self, stdin: SP1Stdin) -> ProofData {
        // Generate the proof. Depending on SP1_PROVER env, this may be a local or network proof.
        let proof = self
            .prover_client
            .prove_groth16(&self.pkey, stdin)
            .expect("proving failed");
        println!("Successfully generated proof!");

        // Verify proof.
        self.prover_client
            .verify_groth16(&proof, &self.vkey)
            .expect("Verification failed");

        let proof_abi = Groth16Proof {
            a: [
                Uint::from_str(&proof.proof.a[0]).unwrap(),
                Uint::from_str(&proof.proof.a[1]).unwrap(),
            ],
            b: [
                [
                    Uint::from_str(&proof.proof.b[0][0]).unwrap(),
                    Uint::from_str(&proof.proof.b[0][1]).unwrap(),
                ],
                [
                    Uint::from_str(&proof.proof.b[1][0]).unwrap(),
                    Uint::from_str(&proof.proof.b[1][1]).unwrap(),
                ],
            ],
            c: [
                Uint::from_str(&proof.proof.c[0]).unwrap(),
                Uint::from_str(&proof.proof.c[1]).unwrap(),
            ],
        }
        .abi_encode();

        ProofData {
            proof: proof_abi,
            pv: proof.public_values.buffer.data,
        }
    }
}

pub struct MockTendermintProver {
    prover: MockProver,
}

impl MockTendermintProver {
    pub fn new(prover: MockProver) -> Self {
        Self { prover }
    }
}

#[async_trait]
impl TendermintProver for MockTendermintProver {
    async fn generate_proof(&self, stdin: SP1Stdin) -> ProofData {
        let proof = self.prover.prove_groth16(TENDERMINT_ELF, stdin).unwrap();

        ProofData {
            proof: proof.proof.to_vec(),
            pv: proof.public_values.buffer.data,
        }
    }
}
