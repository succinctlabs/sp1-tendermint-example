#![allow(dead_code)]
use crate::types::*;
use anyhow::Result;
use reqwest::Client;
use std::{collections::HashMap, env};
use subtle_encoding::hex;
use tendermint::{
    block::signed_header::SignedHeader,
    node::Id,
    validator::{Info, Set},
};
use tendermint_light_client_verifier::types::{LightBlock, ValidatorSet};

pub struct TendermintRPCClient {
    url: String,
    client: Client,
}

impl Default for TendermintRPCClient {
    fn default() -> Self {
        Self::new(env::var("TENDERMINT_RPC_URL").expect("TENDERMINT_RPC_URL not set"))
    }
}

impl TendermintRPCClient {
    pub fn new(url: String) -> Self {
        TendermintRPCClient {
            url,
            client: Client::new(),
        }
    }

    /// Retrieves light blocks for the trusted and target block heights.
    pub async fn get_light_blocks(
        &self,
        trusted_block_height: u64,
        target_block_height: u64,
    ) -> (LightBlock, LightBlock) {
        let peer_id = self.get_peer_id().await.unwrap();

        let trusted_light_block = self
            .fetch_light_block(trusted_block_height, peer_id)
            .await
            .expect("Failed to generate light block 1");
        let target_light_block = self
            .fetch_light_block(target_block_height, peer_id)
            .await
            .expect("Failed to generate light block 2");
        (trusted_light_block, target_light_block)
    }

    /// Retrieves the latest block height from the Tendermint node.
    pub async fn get_latest_block_height(&self) -> u64 {
        let latest_commit = self.fetch_latest_commit().await.unwrap();
        latest_commit.result.signed_header.header.height.value()
    }

    /// Retrieves the block height from a given block hash.
    pub async fn get_block_height_from_hash(&self, hash: &[u8]) -> u64 {
        let block = self.get_block_by_hash(hash).await.unwrap();
        block.result.block.header.height.value()
    }

    /// Fetches a block by its hash.
    async fn get_block_by_hash(&self, hash: &[u8]) -> Result<BlockResponse> {
        let block_by_hash_url = format!(
            "{}/block_by_hash?hash=0x{}",
            self.url,
            String::from_utf8(hex::encode(hash)).unwrap()
        );
        let response: BlockResponse = self
            .client
            .get(block_by_hash_url)
            .send()
            .await?
            .json::<BlockResponse>()
            .await?;
        Ok(response)
    }

    /// Sorts the signatures in the signed header based on the descending order of validators' power.
    fn sort_signatures_by_validators_power_desc(
        &self,
        signed_header: &mut SignedHeader,
        validators_set: &ValidatorSet,
    ) {
        let validator_powers: HashMap<_, _> = validators_set
            .validators()
            .iter()
            .map(|v| (v.address, v.power()))
            .collect();

        signed_header.commit.signatures.sort_by(|a, b| {
            let power_a = a
                .validator_address()
                .and_then(|addr| validator_powers.get(&addr))
                .unwrap_or(&0);
            let power_b = b
                .validator_address()
                .and_then(|addr| validator_powers.get(&addr))
                .unwrap_or(&0);
            power_b.cmp(power_a)
        });
    }

    /// Fetches the peer ID from the Tendermint node.
    async fn get_peer_id(&self) -> Result<[u8; 20]> {
        let client = Client::new();
        let fetch_peer_id_url = format!("{}/status", self.url);

        let response: PeerIdResponse = client
            .get(fetch_peer_id_url)
            .send()
            .await?
            .json::<PeerIdResponse>()
            .await?;

        Ok(hex::decode(response.result.node_info.id)
            .unwrap()
            .try_into()
            .unwrap())
    }

    /// Fetches a light block by its hash.
    async fn get_light_block_by_hash(&self, hash: &[u8]) -> LightBlock {
        let block = self.get_block_by_hash(hash).await.unwrap();
        let peer_id = self.get_peer_id().await.unwrap();
        self.fetch_light_block(
            block.result.block.header.height.value(),
            hex::decode(peer_id).unwrap().try_into().unwrap(),
        )
        .await
        .unwrap()
    }

    /// Fetches the latest commit from the Tendermint node.
    pub async fn fetch_latest_commit(&self) -> Result<CommitResponse> {
        let url = format!("{}/commit", self.url);
        let response: CommitResponse = self
            .client
            .get(url)
            .send()
            .await?
            .json::<CommitResponse>()
            .await?;
        Ok(response)
    }

    /// Get a commit for a specific block height.
    pub async fn get_commit(&self, block_height: u64) -> Result<CommitResponse> {
        let url = format!("{}/{}", self.url, "commit");

        let response: CommitResponse = self
            .client
            .get(url)
            .query(&[
                ("height", block_height.to_string().as_str()),
                ("per_page", "100"), // helpful only when fetching validators
            ])
            .send()
            .await?
            .json::<CommitResponse>()
            .await?;
        Ok(response)
    }

    /// Get validators for a specific block height.
    async fn get_validators(&self, block_height: u64) -> Result<Vec<Info>> {
        let url = format!("{}/{}", self.url, "validators");

        let mut validators = vec![];
        let mut collected_validators = 0;
        let mut page_index = 1;
        loop {
            let response = self
                .client
                .get(&url)
                .query(&[
                    ("height", block_height.to_string().as_str()),
                    ("per_page", "100"),
                    ("page", page_index.to_string().as_str()),
                ])
                .send()
                .await?
                .json::<ValidatorSetResponse>()
                .await?;
            let block_validator_set: BlockValidatorSet = response.result;
            validators.extend(block_validator_set.validators);
            collected_validators += block_validator_set.count.parse::<i32>().unwrap();

            if collected_validators >= block_validator_set.total.parse::<i32>().unwrap() {
                break;
            }
            page_index += 1;
        }

        Ok(validators)
    }

    /// Fetches a light block for a specific block height and peer ID.
    async fn fetch_light_block(&self, block_height: u64, peer_id: [u8; 20]) -> Result<LightBlock> {
        let commit_response = self.get_commit(block_height).await?;
        let mut signed_header = commit_response.result.signed_header;

        let validator_response = self.get_validators(block_height).await?;

        let validators = Set::new(validator_response, None);

        let next_validator_response = self.get_validators(block_height + 1).await?;
        let next_validators = Set::new(next_validator_response, None);

        self.sort_signatures_by_validators_power_desc(&mut signed_header, &validators);
        Ok(LightBlock::new(
            signed_header,
            validators,
            next_validators,
            Id::new(peer_id),
        ))
    }
}
