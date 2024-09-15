use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub enum ChainError {
    BlockHasWrongDifficultyValue(usize),
    BlockHasWrongHashValue(usize),
    BlockHasWrongPreviousHashValue(usize),
    BlockHasWrongSignatureValue(usize),
    BlockHasWrongValidatorValue(usize),
    BlockIsNotNextBlockInSequence(usize),
    ChainHasInvalidGenesisBlock,
    ChainSizeIsNotLongerThanLocalChain,
    NotQualifiedForAgent(String),
    WrongAgentIdentifier(String),
    WrongAgentKey(String),
}

fn error_message(error: &ChainError, f: &mut Formatter<'_>) -> std::fmt::Result {
    match error {
        ChainError::BlockHasWrongDifficultyValue(block_id) => {
            write!(f, "Block {block_id} has invalid difficulty")
        }
        ChainError::BlockHasWrongHashValue(block_id) => {
            write!(f, "Block {block_id} has invalid hash")
        }
        ChainError::BlockHasWrongPreviousHashValue(block_id) => {
            write!(f, "Block {block_id} has invalid previous hash")
        }
        ChainError::BlockHasWrongSignatureValue(block_id) => {
            write!(f, "Block {block_id} has invalid signature")
        }
        ChainError::BlockHasWrongValidatorValue(block_id) => {
            write!(f, "Block {block_id} has invalid validator")
        }
        ChainError::BlockIsNotNextBlockInSequence(block_id) => {
            write!(f, "Block {block_id} is not the next block in the sequence")
        }
        ChainError::ChainHasInvalidGenesisBlock => {
            write!(f, "Chain has invalid genesis block")
        }
        ChainError::ChainSizeIsNotLongerThanLocalChain => {
            write!(f, "Chain size is not longer than local chain")
        }
        ChainError::NotQualifiedForAgent(identifier) => {
            write!(f, "Item with id {identifier} is not qualified to be an agent")
        }
        ChainError::WrongAgentIdentifier(identifier) => {
            write!(f, "Agent with identifier {identifier} does not exist or is not valid")
        }
        ChainError::WrongAgentKey(node) => {
            write!(f, "Agent must have key column defined, but node {node} does not have it")
        }
    }
}

impl Display for ChainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_message(self, f)
    }
}

impl Debug for ChainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_message(self, f)
    }
}
