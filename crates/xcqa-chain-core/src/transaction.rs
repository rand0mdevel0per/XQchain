use serde::{Serialize, Deserialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use xcqa_crypto::{
    MlDsa65Signature, LatticeCommitment, LatticeRangeProof,
    mldsa_verify, CommitmentMatrix,
};
use crate::error::{CoreError, Result};

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct Transaction {
    pub sender: [u8; 32],
    pub recipient: [u8; 32],
    pub amount_commitment: LatticeCommitment,
    pub range_proof: LatticeRangeProof,
    pub nonce: u64,
    pub signature: MlDsa65Signature,
}

impl Transaction {
    pub fn hash(&self) -> [u8; 64] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.sender);
        bytes.extend_from_slice(&self.recipient);
        bytes.extend_from_slice(&rkyv::to_bytes::<rkyv::rancor::Error>(&self.amount_commitment).unwrap());
        bytes.extend_from_slice(&rkyv::to_bytes::<rkyv::rancor::Error>(&self.range_proof).unwrap());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        xcqa_crypto::blake3_512(&bytes)
    }

    pub fn verify(&self, sender_pk: &xcqa_crypto::MlDsa65PublicKey, matrix: &CommitmentMatrix, block_hash: &[u8; 64]) -> Result<()> {
        let msg = self.hash();
        if !mldsa_verify(&msg, &self.signature, sender_pk) {
            return Err(CoreError::InvalidTransaction("Invalid signature".into()));
        }
        if !self.range_proof.verify(&self.amount_commitment, matrix, block_hash) {
            return Err(CoreError::InvalidTransaction("Invalid range proof".into()));
        }
        Ok(())
    }
}

