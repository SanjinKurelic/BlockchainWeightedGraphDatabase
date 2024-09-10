use crate::chain::mine::MiningUtil;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub data: BlockData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Constructor)]
pub struct BlockData {
    pub from: String,
    pub to: String,
    pub weight: i8,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            id: 0,
            hash: "0000494d137e1631bba301d5acab6e7bb7aa74ce1185d456565ef51d737677b2".to_string(),
            previous_hash: "".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            nonce: 2024,
            data: BlockData {
                from: "".to_string(),
                to: "".to_string(),
                weight: 0,
            },
        }
    }
}

impl Block {
    pub fn new(id: u64, previous_hash: String, data: BlockData) -> Block {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let (nonce, hash) = MiningUtil::mine_block(id, timestamp, &previous_hash, &data);

        Block {
            id,
            data,
            hash,
            nonce,
            previous_hash,
            timestamp,
        }
    }
}
