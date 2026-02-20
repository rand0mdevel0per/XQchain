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
}
