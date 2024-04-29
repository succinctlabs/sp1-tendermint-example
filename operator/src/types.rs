use serde::{Deserialize, Serialize};
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
