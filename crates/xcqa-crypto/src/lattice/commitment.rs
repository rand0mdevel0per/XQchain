use super::params::{K, L, N, Q};
use super::ring::{RingElement, RingVector};
use crate::hash::blake3_512;
use rand::{Rng, CryptoRng};
use serde::{Serialize, Deserialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

pub struct CommitmentMatrix {
    pub a: Vec<RingVector>,  // k×l matrix
}

impl CommitmentMatrix {
    pub fn from_genesis(_genesis_hash: &[u8; 64]) -> Self {
        let seed = blake3_512(b"XCQA-CHAIN-COMMIT-MATRIX-V1");
        let mut matrix = Vec::with_capacity(K);

        for i in 0..K {
            let mut row = Vec::with_capacity(L);
            for j in 0..L {
                let mut coeffs = [0u32; N];
                for k in 0..N {
                    let idx = (i * L * N + j * N + k) % seed.len();
                    coeffs[k] = (seed[idx] as u32) % Q;
                }
                row.push(RingElement { coeffs });
            }
            matrix.push(RingVector { elements: row });
        }

        Self { a: matrix }
    }
}

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct LatticeCommitment {
    pub c: RingVector,  // k elements
}

impl LatticeCommitment {
    pub fn commit(value: u64, randomness: &LatticeRandVec, matrix: &CommitmentMatrix) -> Self {
        let mut result = RingVector::zero(K);

        // A·r
        for i in 0..K {
            for j in 0..L {
                let prod = matrix.a[i].elements[j].scalar_mul(randomness.r.elements[j].coeffs[0] as u64);
                result.elements[i] = result.elements[i].add(&prod);
            }
        }

        // + v·G (gadget vector)
        let v_mod = (value % Q as u64) as u32;
        result.elements[0].coeffs[0] = (result.elements[0].coeffs[0] + v_mod) % Q;

        Self { c: result }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self { c: self.c.add(&other.c) }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self { c: self.c.sub(&other.c) }
    }
}

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct LatticeRandVec {
    pub r: RingVector,  // l elements
}

impl LatticeRandVec {
    pub fn random<R: Rng + CryptoRng>(rng: &mut R) -> Self {
        let mut elements = Vec::with_capacity(L);
        for _ in 0..L {
            let mut coeffs = [0u32; N];
            for c in &mut coeffs {
                *c = (rng.next_u32() % 256) as u32;  // Small coefficients
            }
            elements.push(RingElement { coeffs });
        }
        Self { r: RingVector { elements } }
    }
}
