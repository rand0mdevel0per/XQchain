use crate::{Block, BlockHeader, Transaction};
use crate::error::{CoreError, Result};
use std::collections::HashMap;

pub struct Blockchain {
    blocks: Vec<Block>,
    height: u64,
    balances: HashMap<[u8; 32], u64>,
}

impl Blockchain {
    pub fn new(genesis: Block) -> Self {
        Self {
            blocks: vec![genesis],
            height: 0,
            balances: HashMap::new(),
        }
    }

    pub fn genesis() -> Block {
        Block {
            header: BlockHeader {
                height: 0,
                prev_hash: [0u8; 64],
                timestamp: 0,
                difficulty_tier: 0,
                fine_difficulty: 1,
            },
            transactions: vec![],
            xcqa_sig: xcqa_crypto::XcqaSignature {
                commitment: vec![],
                response: vec![],
            },
            xcqa_nonce: [0u8; 32],
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        if block.header.height != self.height + 1 {
            return Err(CoreError::InvalidBlock("Invalid height".into()));
        }

        for tx in &block.transactions {
            let sender_balance = self.balances.get(&tx.sender).copied().unwrap_or(0);
            if sender_balance == 0 {
                return Err(CoreError::InvalidTransaction("Insufficient balance".into()));
            }
        }

        self.height += 1;
        self.blocks.push(block);
        Ok(())
    }

    pub fn latest_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }

    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn calculate_next_difficulty(&self) -> (u8, u8) {
        const TARGET_BLOCK_TIME: u64 = 10;
        const ADJUSTMENT_WINDOW: usize = 10;

        if self.height < 2 {
            return (0, 1);
        }

        let window_size = ADJUSTMENT_WINDOW.min(self.height as usize);
        let recent_blocks = &self.blocks[self.blocks.len() - window_size..];

        let time_diff = recent_blocks.last().unwrap().header.timestamp
                      - recent_blocks.first().unwrap().header.timestamp;
        let avg_time = time_diff / (window_size as u64 - 1).max(1);

        let current = self.latest_block().header;
        let (mut tier, mut fine) = (current.difficulty_tier, current.fine_difficulty);

        if avg_time < TARGET_BLOCK_TIME * 8 / 10 {
            fine = (fine + 1).min(8);
            if fine == 8 { tier += 1; fine = 1; }
        } else if avg_time > TARGET_BLOCK_TIME * 12 / 10 {
            if fine > 1 { fine -= 1; }
            else if tier > 0 { tier -= 1; fine = 8; }
        }

        (tier, fine)
    }
}
