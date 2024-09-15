use crate::chain::error::ChainError;
use crate::chain::wallet::Wallet;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::time::{SystemTime, UNIX_EPOCH};
use rustc_hash::FxHashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Block {
    pub id: usize,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: BlockData,
    pub validator: String,
    pub signature: String,
    pub difficulty: usize,
}

#[derive(Serialize, Deserialize, Constructor, Clone, PartialEq)]
pub struct BlockData {
    pub data_type: BlockDataType,
    pub edge_data: Option<EdgeData>,
    pub validator_data: Option<ValidatorData>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum BlockDataType {
    EdgeData,
    ValidatorData,
    RootNode,
}

#[derive(Serialize, Deserialize, Constructor, Clone, PartialEq)]
pub struct EdgeData {
    pub from: String,
    pub to: String,
    pub weight: i8,
}

#[derive(Serialize, Deserialize, Constructor, Clone, PartialEq)]
pub struct ValidatorData {
    pub public_key: String,
    pub account_id: String,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            id: 0,
            hash: "0000494d137e1631bba301d5acab6e7bb7aa74ce1185d456565ef51d737677b2".to_string(),
            previous_hash: "".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data: BlockData::new(BlockDataType::RootNode, None, None),
            validator: "".to_string(),
            signature: "".to_string(),
            difficulty: 0,
        }
    }
}

impl Block {
    pub fn new(id: usize, previous_hash: String, data: BlockData, wallet: &mut Wallet, difficulty: usize) -> Block {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let validator = wallet.get_public_key();
        let hash = Block::calculate_hash(id, timestamp, &previous_hash, &data, &validator, difficulty);

        Block {
            id,
            data,
            hash: hash.clone(),
            previous_hash,
            timestamp,
            validator,
            signature: wallet.sign(&hash),
            difficulty,
        }
    }

    pub fn validate_block_hash(block: &Block) -> Result<(), ChainError> {
        let hash = Block::calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            &block.validator,
            block.difficulty,
        );

        if hash != block.hash {
            return Err(ChainError::BlockHasWrongHashValue(block.id));
        }

        Ok(())
    }

    fn calculate_hash(id: usize, timestamp: u64, previous_hash: &str, data: &BlockData, validator: &String, difficulty: usize) -> String {
        digest(
            serde_json::json!({
                "id": id,
                "timestamp": timestamp,
                "previous_hash": previous_hash,
                "data": data,
                "validator": validator,
                "difficulty": difficulty,
            })
            .to_string(),
        )
    }

    pub fn as_hash_map(&self) -> FxHashMap<String, String> {
        let mut map = FxHashMap::default();

        map.insert("id".to_string(), self.id.to_string());
        map.insert("hash".to_string(), self.hash.clone());
        map.insert("previous_hash".to_string(), self.previous_hash.clone());
        map.insert("timestamp".to_string(), self.timestamp.to_string());
        map.insert("data".to_string(), serde_json::to_string(&self.data).unwrap());
        map.insert("validator".to_string(), self.validator.clone());
        map.insert("signature".to_string(), self.signature.clone());
        map.insert("difficulty".to_string(), self.difficulty.to_string());

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_calculate_hash() {
        // Given
        let block_data = BlockData::new(
            BlockDataType::ValidatorData,
            None,
            Some(ValidatorData::new("public_key".to_string(), "account_id".to_string())),
        );
        let block = Block::new(1, "previous_hash".to_string(), block_data, &mut Wallet::default(), 0);

        // When
        let hash = Block::calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            &block.validator,
            block.difficulty,
        );

        // Then
        assert_eq!(block.hash, hash);
        assert!(Block::validate_block_hash(&block).is_ok());
    }
}
