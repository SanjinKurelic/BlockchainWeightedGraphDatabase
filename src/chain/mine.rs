use crate::chain::block::BlockData;
use sha256::digest;

pub struct MiningUtil;

impl MiningUtil {
    const DIFFICULTY_PREFIX: &'static str = "00";

    pub fn calculate_hash(id: u64, timestamp: u64, previous_hash: &str, data: &BlockData, nonce: u64) -> String {
        digest(format!(
            r#"
            {{
                "id": {id},
                "data": {{
                    "from": {},
                    "to": {},
                    "weight": {}
                }},
                "nonce": {nonce},
                "previous_hash": {previous_hash},
                "timestamp": {timestamp}
            }}
            "#,
            data.from, data.to, data.weight
        ))
    }

    pub fn mine_block(id: u64, timestamp: u64, previous_hash: &str, data: &BlockData) -> (u64, String) {
        let mut nonce = 0;

        loop {
            let hash = Self::calculate_hash(id, timestamp, previous_hash, data, nonce);

            if Self::has_valid_difficulty(&hash) {
                return (nonce, hex::encode(hash));
            }

            nonce += 1;
        }
    }

    pub fn has_valid_difficulty(hash: &String) -> bool {
        let binary = hash.clone().into_bytes().iter().map(|&core| format!("{core:b}")).collect();

        binary.starts_with(Self::DIFFICULTY_PREFIX)
    }
}
