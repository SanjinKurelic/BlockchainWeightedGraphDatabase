use crate::chain::agent::AgentService;
use crate::chain::block::{Block, BlockData, BlockDataType, EdgeData, ValidatorData};
use crate::chain::wallet::Wallet;
use crate::graph::{Graph, GraphResults};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use error::ChainError;
use rustc_hash::FxHashMap;
use std::str::FromStr;

mod agent;
pub mod block;
pub(crate) mod error;
mod wallet;

pub struct Chain {
    pub blocks: Vec<Block>,
    pub(crate) wallet: Wallet,
    pub(crate) agent_service: AgentService,
}

impl Default for Chain {
    fn default() -> Self {
        Chain {
            blocks: vec![Block::default()],
            wallet: Wallet::default(),
            agent_service: AgentService::default(),
        }
    }
}

impl Chain {
    pub fn define_agent(&mut self, node_name: String, conditions: FxHashMap<String, String>) {
        self.agent_service.define_agent(node_name, conditions)
    }

    pub fn add_or_update_agent(&mut self, graph: &mut Graph, node_name: String, identifier: String) -> Result<(), ChainError> {
        let (p_key, difficulty) = self.agent_service.add_or_update_agent(graph, node_name, &identifier)?;

        if p_key == self.wallet.get_public_key() {
            let validator_data = ValidatorData::new(self.wallet.get_public_key(), identifier.clone());
            let block_data = BlockData::new(BlockDataType::ValidatorData, None, Some(validator_data));

            let block = Block::new(
                self.blocks.len(),
                self.blocks.last().unwrap().hash.clone(),
                block_data,
                &mut self.wallet,
                difficulty,
            );

            self.add_new_block(block)?
        }

        Ok(())
    }

    pub fn remove_agent(&mut self, identifier: String) {
        self.agent_service.remove_agent(&identifier);
    }

    pub fn add_edge_change(&mut self, from: String, to: String, weight: i8) -> Result<(), ChainError> {
        let data = EdgeData::new(from.clone(), to, weight);

        let block = Block::new(
            self.blocks.len(),
            self.blocks.last().unwrap().hash.clone(),
            BlockData::new(BlockDataType::EdgeData, Some(data), None),
            &mut self.wallet,
            self.agent_service.get_difficulty(&from),
        );

        self.add_new_block(block)
    }

    pub fn replace_chain(&mut self, chain: &Vec<Block>) -> Result<(), ChainError> {
        self.validate_chain(chain)?;

        if chain.len() <= self.blocks.len() {
            return Err(ChainError::ChainSizeIsNotLongerThanLocalChain);
        }

        self.blocks = chain.clone();

        Ok(())
    }

    pub fn add_new_block(&mut self, block: Block) -> Result<(), ChainError> {
        let previous_block = self.blocks.last().unwrap();

        self.validate_block(&block, previous_block)?;

        self.blocks.push(block);

        Ok(())
    }

    fn validate_block(&self, block: &Block, previous_block: &Block) -> Result<(), ChainError> {
        if block.previous_hash != previous_block.hash {
            return Err(ChainError::BlockHasWrongPreviousHashValue(block.id));
        }

        if block.id != previous_block.id + 1 {
            return Err(ChainError::BlockIsNotNextBlockInSequence(block.id));
        }

        Block::validate_block_hash(block)?;
        self.validate_signature(block.id, &block.validator, &block.signature, &block.hash)?;
        self.validate_stake(block.id, &block.validator, block.difficulty)?;

        Ok(())
    }

    fn validate_chain(&self, chain: &[Block]) -> Result<(), ChainError> {
        if *chain.first().unwrap() != Block::default() {
            return Err(ChainError::ChainHasInvalidGenesisBlock);
        }

        for i in 1..chain.len() {
            let block = &chain[i];
            let previous_block = &chain[i - 1];

            if previous_block.hash != block.previous_hash {
                return Err(ChainError::BlockHasWrongPreviousHashValue(block.id));
            } else if previous_block.id + 1 != block.id {
                return Err(ChainError::BlockIsNotNextBlockInSequence(block.id));
            }
        }

        Ok(())
    }

    fn validate_signature(&self, id: usize, validator: &String, signature: &String, hash: &String) -> Result<(), ChainError> {
        let public_key = VerifyingKey::from_bytes(
            hex::decode(validator)
                .map_err(|_| ChainError::BlockHasWrongValidatorValue(id))?
                .as_slice()
                .try_into()
                .map_err(|_| ChainError::BlockHasWrongValidatorValue(id))?,
        )
        .map_err(|_| ChainError::BlockHasWrongValidatorValue(id))?;

        Ok(public_key
            .verify(
                hash.as_bytes(),
                &Signature::from_str(signature.as_str()).map_err(|_| ChainError::BlockHasWrongSignatureValue(id))?,
            )
            .map_err(|_| ChainError::BlockHasWrongSignatureValue(id))?)
    }

    fn validate_stake(&self, id: usize, validator: &String, difficulty: usize) -> Result<(), ChainError> {
        let validator_difficulty = self.agent_service.get_validator_difficulty(validator);

        if validator_difficulty < difficulty {
            return Err(ChainError::BlockHasWrongDifficultyValue(id));
        }

        Ok(())
    }

    pub fn as_graph_result(&self) -> GraphResults{
        Ok(self.blocks.iter().map(|block| block.as_hash_map()).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::attribute::InternalNodeAttribute;
    use super::*;
    use crate::graph::node::Node;

    #[test]
    fn should_define_agent() {
        // Given
        let mut chain = Chain::default();

        // When
        chain.define_agent("User".to_string(), FxHashMap::default());

        // Then
        assert_eq!(chain.agent_service.agents.len(), 1);
    }

    #[test]
    fn should_add_or_update_agent() {
        // Given
        let mut chain = Chain::default();
        let mut graph = Graph::default();

        let mut attributes = FxHashMap::default();
        attributes.insert(InternalNodeAttribute::ID_ATTRIBUTE.to_string(), "identifier".to_string());
        attributes.insert("key".to_string(), chain.wallet.get_public_key());
        graph.nodes.insert("identifier:User".to_string(), Node::new(attributes, vec![]));

        chain.agent_service.agents.insert("User".to_string(), FxHashMap::default());

        // When
        let result = chain.add_or_update_agent(&mut graph, "User".to_string(), "identifier".to_string());

        // Then
        assert!(result.is_ok());
        assert_eq!(chain.agent_service.agents.len(), 1);
        assert_eq!(chain.blocks.len(), 2);
    }

    #[test]
    fn should_add_edge_change() {
        // Given
        let mut chain = Chain::default();

        // When
        let result = chain.add_edge_change("from".to_string(), "to".to_string(), 1);

        // Then
        assert!(result.is_ok());
        assert_eq!(chain.blocks.len(), 2);
        assert_block(
            chain.blocks.last().unwrap(),
            Some(EdgeData::new("from".to_string(), "to".to_string(), 1)),
            None,
        );
    }

    #[test]
    fn should_replace_chain() {}

    #[test]
    fn should_add_new_block() {
        // Given
        let mut chain = Chain::default();
        let previous_block = chain.blocks.last().unwrap().clone();

        // When
        let result = chain.add_new_block(Block::new(
            chain.blocks.len(),
            previous_block.hash.clone(),
            BlockData::new(
                BlockDataType::ValidatorData,
                None,
                Some(ValidatorData::new("public_key".to_string(), "account_id".to_string())),
            ),
            &mut Wallet::default(),
            0,
        ));

        // Then
        assert!(result.is_ok());
        assert_eq!(chain.blocks.len(), 2);
        assert_block(
            chain.blocks.last().unwrap(),
            None,
            Some(ValidatorData::new("public_key".to_string(), "account_id".to_string())),
        );
    }

    fn assert_block(block: &Block, edge_data: Option<EdgeData>, validator_data: Option<ValidatorData>) {
        assert_eq!(block.id, 1);
        assert_eq!(block.previous_hash, "0000494d137e1631bba301d5acab6e7bb7aa74ce1185d456565ef51d737677b2");

        assert!(block.timestamp > 0);
        assert!(!block.hash.is_empty());
        assert!(!block.validator.is_empty());
        assert!(!block.signature.is_empty());

        assert_eq!(block.difficulty, 0);
        assert!(
            block.data.data_type
                == if edge_data.is_none() {
                    BlockDataType::ValidatorData
                } else {
                    BlockDataType::EdgeData
                }
        );
        assert!(block.data.edge_data == edge_data);
        assert!(block.data.validator_data == validator_data);
    }
}
