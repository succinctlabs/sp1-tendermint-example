#![allow(dead_code)]
use std::collections::HashMap;
use std::error::Error;

use reqwest::Client;
use serde::Deserialize;
use subtle_encoding::hex;
use tendermint::{
    block::{self, signed_header::SignedHeader},
    node::Id,
    validator::{Info, Set},
    Block,
};
use tendermint_light_client_verifier::types::{LightBlock, ValidatorSet};

#[derive(Debug, Deserialize)]
pub struct PeerIdResponse {
    pub result: PeerIdWrapper,
}

#[derive(Debug, Deserialize)]
pub struct PeerIdWrapper {
    pub node_info: NodeInfoWrapper,
}

#[derive(Debug, Deserialize)]
pub struct NodeInfoWrapper {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockResponse {
    pub result: BlockWrapper,
}

#[derive(Debug, Deserialize)]
pub struct BlockWrapper {
    pub block_id: Option<block::Id>,
    pub block: Block,
}

#[derive(Debug, Deserialize)]
pub struct CommitResponse {
    pub result: SignedHeaderWrapper,
}

#[derive(Debug, Deserialize)]
pub struct SignedHeaderWrapper {
    pub signed_header: SignedHeader,
}

#[derive(Debug, Deserialize)]
pub struct ValidatorSetResponse {
    pub result: BlockValidatorSet,
}

#[derive(Debug, Deserialize)]
pub struct BlockValidatorSet {
    pub block_height: String,
    pub validators: Vec<Info>,
    pub count: String,
    pub total: String,
}

pub fn sort_signatures_by_validators_power_desc(
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

pub async fn fetch_peer_id(client: &Client, url: &str) -> Result<PeerIdResponse, Box<dyn Error>> {
    let response: PeerIdResponse = client
        .get(url)
        .send()
        .await?
        .json::<PeerIdResponse>()
        .await?;
    Ok(response)
}

pub async fn fetch_block(client: &Client, url: &str) -> Result<BlockResponse, Box<dyn Error>> {
    let response: BlockResponse = client
        .get(url)
        .send()
        .await?
        .json::<BlockResponse>()
        .await?;
    Ok(response)
}

pub async fn get_light_blocks(
    trusted_header_hash: &[u8],
    target_block_height: u64,
) -> (LightBlock, LightBlock) {
    const BASE_URL: &str = "https://celestia-mocha-rpc.publicnode.com:443";

    let fetch_peer_id_url = format!("{}/status", BASE_URL);

    let client = Client::new();

    let peer_id_response = fetch_peer_id(&client, &fetch_peer_id_url).await.unwrap();
    let peer_id_str = peer_id_response.result.node_info.id;
    let peer_id = hex::decode(peer_id_str).unwrap();
    let peer_id = peer_id.try_into().unwrap();

    let block_by_hash_url = format!(
        "{}/block_by_hash?hash=0x{}",
        BASE_URL,
        String::from_utf8(hex::encode(trusted_header_hash)).unwrap()
    );

    let trusted_block = fetch_block(&client, &block_by_hash_url).await.unwrap();
    let trusted_height = trusted_block.result.block.header.height.value();

    let trusted_light_block = fetch_light_block(trusted_height, peer_id, BASE_URL)
        .await
        .expect("Failed to generate light block 1");
    let target_light_block = fetch_light_block(target_block_height, peer_id, BASE_URL)
        .await
        .expect("Failed to generate light block 2");
    (trusted_light_block, target_light_block)
}

pub async fn get_light_block_by_hash(hash: &[u8]) -> LightBlock {
    let peer_id: [u8; 20] = [
        0x72, 0x6b, 0xc8, 0xd2, 0x60, 0x38, 0x7c, 0xf5, 0x6e, 0xcf, 0xad, 0x3a, 0x6b, 0xf6, 0xfe,
        0xcd, 0x90, 0x3e, 0x18, 0xa2,
    ];
    const BASE_URL: &str = "https://celestia-mocha-rpc.publicnode.com:443";

    let url = format!(
        "{}/block_by_hash?hash=0x{}",
        BASE_URL,
        String::from_utf8(hex::encode(hash)).unwrap()
    );
    let client = Client::new();
    let block = fetch_block(&client, &url).await.unwrap();
    fetch_light_block(block.result.block.header.height.value(), peer_id, BASE_URL)
        .await
        .unwrap()
}

pub async fn get_latest_block_height() -> u64 {
    let url = "https://celestia-mocha-rpc.publicnode.com:443/commit";
    let client = Client::new();
    let latest_commit = fetch_latest_commit(&client, url).await.unwrap();
    latest_commit.result.signed_header.header.height.value()
}

pub async fn fetch_latest_commit(
    client: &Client,
    url: &str,
) -> Result<CommitResponse, Box<dyn Error>> {
    let response: CommitResponse = client
        .get(url)
        .send()
        .await?
        .json::<CommitResponse>()
        .await?;
    Ok(response)
}

pub async fn fetch_commit(
    client: &Client,
    url: &str,
    block_height: u64,
) -> Result<CommitResponse, Box<dyn Error>> {
    let response: CommitResponse = client
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

pub async fn fetch_validators(
    client: &Client,
    url: &str,
    block_height: u64,
) -> Result<Vec<Info>, Box<dyn Error>> {
    let mut validators = vec![];
    let mut collected_validators = 0;
    let mut page_index = 1;
    loop {
        let response = client
            .get(url)
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

pub async fn fetch_light_block(
    block_height: u64,
    peer_id: [u8; 20],
    base_url: &str,
) -> Result<LightBlock, Box<dyn Error>> {
    let client = Client::new();

    let commit_response =
        fetch_commit(&client, &format!("{}/commit", base_url), block_height).await?;
    let mut signed_header = commit_response.result.signed_header;

    let validator_response =
        fetch_validators(&client, &format!("{}/validators", base_url), block_height).await?;

    let validators = Set::new(validator_response, None);

    let next_validator_response = fetch_validators(
        &client,
        &format!("{}/validators", base_url),
        block_height + 1,
    )
    .await?;
    let next_validators = Set::new(next_validator_response, None);

    sort_signatures_by_validators_power_desc(&mut signed_header, &validators);
    Ok(LightBlock::new(
        signed_header,
        validators,
        next_validators,
        Id::new(peer_id),
    ))
}
