use super::commitment::{CommitmentMatrix, LatticeCommitment, LatticeRandVec};
use crate::hash::blake3_512;
use rand::{Rng, CryptoRng};
use serde::{Serialize, Deserialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

const BITS: usize = 64;

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct BitProof {
    commitment: LatticeCommitment,
    response_r: LatticeRandVec,
    response_b: u8,
}

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct LatticeRangeProof {
    bit_commitments: Vec<LatticeCommitment>,
    bit_proofs: Vec<BitProof>,
}

fn decompose_bits(value: u64) -> [u8; BITS] {
    let mut bits = [0u8; BITS];
    for i in 0..BITS {
        bits[i] = ((value >> i) & 1) as u8;
    }
    bits
}

impl LatticeRangeProof {
    pub fn prove<R: Rng + CryptoRng>(
        value: u64,
        _randomness: &LatticeRandVec,
        _commitment: &LatticeCommitment,
        matrix: &CommitmentMatrix,
        rng: &mut R,
        block_hash: &[u8; 64],
    ) -> Self {
        let bits = decompose_bits(value);
        let mut bit_commitments = Vec::with_capacity(BITS);
        let mut bit_randomness = Vec::with_capacity(BITS);

        // Create commitment for each bit
        for &bit in &bits {
            let r = LatticeRandVec::random(rng);
            let c = LatticeCommitment::commit(bit as u64, &r, matrix);
            bit_commitments.push(c);
            bit_randomness.push(r);
        }

        // Generate Fiat-Shamir challenge
        let mut challenge_input = Vec::new();
        for c in &bit_commitments {
            for coeff in &c.c.elements[0].coeffs {
                challenge_input.extend_from_slice(&coeff.to_le_bytes());
            }
        }
        challenge_input.extend_from_slice(block_hash);
        let _challenge = blake3_512(&challenge_input);

        // Generate bit proofs
        let mut bit_proofs = Vec::with_capacity(BITS);
        for i in 0..BITS {
            let proof = BitProof {
                commitment: bit_commitments[i].clone(),
                response_r: bit_randomness[i].clone(),
                response_b: bits[i],
            };
            bit_proofs.push(proof);
        }

        Self { bit_commitments, bit_proofs }
    }

    pub fn verify(
        &self,
        _commitment: &LatticeCommitment,
        matrix: &CommitmentMatrix,
        _block_hash: &[u8; 64],
    ) -> bool {
        if self.bit_commitments.len() != BITS || self.bit_proofs.len() != BITS {
            return false;
        }

        // Verify each bit is 0 or 1
        for proof in &self.bit_proofs {
            if proof.response_b > 1 {
                return false;
            }
            let recomputed = LatticeCommitment::commit(
                proof.response_b as u64,
                &proof.response_r,
                matrix,
            );
            if !commitments_equal(&recomputed, &proof.commitment) {
                return false;
            }
        }

        true
    }

    pub fn compress(&self) -> Vec<u8> {
        let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(self).unwrap();
        zstd::encode_all(&serialized[..], 6).unwrap()
    }

    pub fn decompress(compressed: &[u8]) -> Result<Self, String> {
        let serialized = zstd::decode_all(compressed).map_err(|e| e.to_string())?;
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized).map_err(|e| e.to_string())
    }
}

fn commitments_equal(a: &LatticeCommitment, b: &LatticeCommitment) -> bool {
    a.c.elements.iter().zip(&b.c.elements).all(|(x, y)| {
        x.coeffs.iter().zip(&y.coeffs).all(|(c1, c2)| c1 == c2)
    })
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AmountLeqProof {
    pub diff_commitment: LatticeCommitment,
    pub range_proof: LatticeRangeProof,
}

impl AmountLeqProof {
    pub fn prove<R: Rng + CryptoRng>(
        actual: u64,
        max: u64,
        _actual_rand: &LatticeRandVec,
        _actual_commitment: &LatticeCommitment,
        matrix: &CommitmentMatrix,
        rng: &mut R,
        block_hash: &[u8; 64],
    ) -> Option<Self> {
        if actual > max {
            return None;
        }

        let diff = max - actual;
        let diff_rand = LatticeRandVec::random(rng);
        let diff_commitment = LatticeCommitment::commit(diff, &diff_rand, matrix);

        let range_proof = LatticeRangeProof::prove(
            diff,
            &diff_rand,
            &diff_commitment,
            matrix,
            rng,
            block_hash,
        );

        Some(Self { diff_commitment, range_proof })
    }

    pub fn verify(
        &self,
        _actual_commitment: &LatticeCommitment,
        _max: u64,
        matrix: &CommitmentMatrix,
        block_hash: &[u8; 64],
    ) -> bool {
        self.range_proof.verify(&self.diff_commitment, matrix, block_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_proof_basic() {
        let mut rng = rand::rng();
        let genesis_hash = [0u8; 64];
        let matrix = CommitmentMatrix::from_genesis(&genesis_hash);

        let value = 12345u64;
        let rand = LatticeRandVec::random(&mut rng);
        let commitment = LatticeCommitment::commit(value, &rand, &matrix);

        let block_hash = [1u8; 64];
        let proof = LatticeRangeProof::prove(value, &rand, &commitment, &matrix, &mut rng, &block_hash);

        assert!(proof.verify(&commitment, &matrix, &block_hash));
    }

    #[test]
    fn test_amount_leq_proof() {
        let mut rng = rand::rng();
        let genesis_hash = [0u8; 64];
        let matrix = CommitmentMatrix::from_genesis(&genesis_hash);

        let actual = 100u64;
        let max = 200u64;
        let rand = LatticeRandVec::random(&mut rng);
        let commitment = LatticeCommitment::commit(actual, &rand, &matrix);

        let block_hash = [2u8; 64];
        let proof = AmountLeqProof::prove(actual, max, &rand, &commitment, &matrix, &mut rng, &block_hash);

        assert!(proof.is_some());
        assert!(proof.unwrap().verify(&commitment, max, &matrix, &block_hash));
    }

    #[test]
    fn test_compress_decompress() {
        let mut rng = rand::rng();
        let genesis_hash = [0u8; 64];
        let matrix = CommitmentMatrix::from_genesis(&genesis_hash);

        let value = 12345u64;
        let rand = LatticeRandVec::random(&mut rng);
        let commitment = LatticeCommitment::commit(value, &rand, &matrix);

        let block_hash = [1u8; 64];
        let proof = LatticeRangeProof::prove(value, &rand, &commitment, &matrix, &mut rng, &block_hash);

        let compressed = proof.compress();
        let decompressed = LatticeRangeProof::decompress(&compressed).unwrap();

        assert!(decompressed.verify(&commitment, &matrix, &block_hash));
    }
}
