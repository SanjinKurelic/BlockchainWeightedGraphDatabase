use crate::chain::block::Block;
use crate::chain::mine::MiningUtil;
use error::ChainError;

pub mod block;
mod error;
mod mine;

pub struct Chain {
    pub blocks: Vec<Block>,
}

impl Default for Chain {
    fn default() -> Self {
        Chain {
            blocks: vec![Block::default()],
        }
    }
}

impl Chain {
    pub fn add_block(&mut self, block: Block) -> Result<(), ChainError> {
        let previous_block = self.blocks.last().unwrap();

        self.validate_block(&block, previous_block)?;

        self.blocks.push(block);

        Ok(())
    }

    fn validate_block(&self, block: &Block, previous_block: &Block) -> Result<(), ChainError> {
        if block.previous_hash != previous_block.hash {
            return Err(ChainError::BlockHasWrongPreviousHash(block.id));
        }

        if block.id != previous_block.id + 1 {
            return Err(ChainError::BlockIsNotNextBlockInSequence(block.id));
        }

        if hex::encode(MiningUtil::calculate_hash(block.id, block.timestamp, &block.previous_hash, &block.data, block.nonce)) != block.hash {
            return Err(ChainError::BlockHasWrongHash(block.id));
        }

        if !MiningUtil::has_valid_difficulty(&String::from_utf8(hex::decode(&block.hash).unwrap()).unwrap()) {
            return Err(ChainError::BlockHasInvalidDifficulty(block.id));
        }

        Ok(())
    }

    fn validate_chain(&self, chain: &[Block]) -> Result<(), ChainError> {
        for i in 1..chain.len() {
            self.validate_block(chain.get(i - 1).unwrap(), chain.get(i).unwrap())?
        }

        Ok(())
    }

    fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Result<Vec<Block>, ChainError> {
        let is_local_valid = self.validate_chain(&local);
        let is_remote_valid = self.validate_chain(&remote);

        match (is_local_valid, is_remote_valid) {
            (Ok(_), Ok(_)) => {
                if local.len() > remote.len() {
                    Ok(local)
                } else {
                    Ok(remote)
                }
            }
            (Err(_), Ok(_)) => Ok(remote),
            (Ok(_), Err(_)) => Ok(local),
            (_, _) => Err(ChainError::NoValidChainFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::block::BlockData;
    use super::*;

    #[test]
    fn should_add_block() {
        // Given
        let mut chain = Chain::default();
        let block_data = BlockData::new("from".to_string(), "to".to_string(), 75);
        let block = Block::new(1, chain.blocks.last().unwrap().hash.clone(), block_data);

        // When
        let result = chain.add_block(block);

        // Then
        assert!(result.is_ok());
        assert_eq!(chain.blocks.len(), 2);
    }
}