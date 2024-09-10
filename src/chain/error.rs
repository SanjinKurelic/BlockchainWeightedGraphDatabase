pub enum ChainError {
    BlockHasInvalidDifficulty(u64),
    BlockHasWrongHash(u64),
    BlockHasWrongPreviousHash(u64),
    BlockIsNotNextBlockInSequence(u64),
    NoValidChainFound,
}
