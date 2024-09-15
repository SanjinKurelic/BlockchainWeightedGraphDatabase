use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use crate::chain::block::Block;

#[derive(Serialize, Deserialize)]
pub struct ChainRequest {
    pub from_peer: PeerId,
}

#[derive(Serialize, Deserialize)]
pub struct ChainResponse {
    pub chain: Vec<Block>,
    pub candidates: Vec<Block>,
    pub to_peer: PeerId,
}