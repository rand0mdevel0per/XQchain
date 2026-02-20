use serde::{Serialize, Deserialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use xcqa_crypto::{XcqaSignature, XcqaPublicKeyWrapped, xcqa_verify, CommitmentMatrix};
use crate::transaction::Transaction;
use crate::error::{CoreError, Result};

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct BlockHeader {
    pub height: u64,
    #[serde(with = "serde_big_array::BigArray")]
    pub prev_hash: [u8; 64],
    pub timestamp: u64,
    pub difficulty_tier: u8,
    pub fine_difficulty: u8,
}

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub xcqa_sig: XcqaSignature,
    #[serde(with = "serde_big_array::BigArray")]
    pub xcqa_nonce: [u8; 32],
}

impl Block {
    pub fn hash(&self) -> [u8; 64] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.height.to_le_bytes());
        bytes.extend_from_slice(&self.header.prev_hash);
        bytes.extend_from_slice(&self.header.timestamp.to_le_bytes());
        bytes.push(self.header.difficulty_tier);
        bytes.push(self.header.fine_difficulty);
        xcqa_crypto::blake3_512(&bytes)
    }

    pub fn verify_pow(&self, epoch_pk: &XcqaPublicKeyWrapped) -> Result<()> {
        let block_hash = self.hash();
        let msg = [&block_hash[..], &self.xcqa_nonce[..]].concat();
        if !xcqa_verify(&msg, &self.xcqa_sig, epoch_pk, &block_hash) {
            return Err(CoreError::InvalidBlock("Invalid PoW signature".into()));
        }
        Ok(())
    }

    pub fn verify_transactions(&self, sender_pks: &[xcqa_crypto::MlDsa65PublicKey], matrix: &CommitmentMatrix) -> Result<()> {
        let block_hash = self.hash();
        if sender_pks.len() != self.transactions.len() {
            return Err(CoreError::InvalidBlock("Mismatched public keys count".into()));
        }
        for (tx, pk) in self.transactions.iter().zip(sender_pks.iter()) {
            tx.verify(pk, matrix, &block_hash)?;
        }
        Ok(())
    }
}
