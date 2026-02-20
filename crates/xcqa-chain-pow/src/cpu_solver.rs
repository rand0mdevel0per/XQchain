use xcqa_crypto::{generate_epoch_pk, xcqa_keygen_privacy, xcqa_sign, XcqaSignature};
use crate::difficulty::check_difficulty;
use crate::error::Result;
use rand::{Rng, RngExt};

pub struct CpuSolver {
    layer_count: usize,
}

impl CpuSolver {
    pub fn new(layer_count: usize) -> Self {
        Self { layer_count }
    }

    pub fn mine(
        &self,
        block_header: &[u8],
        block_hash: &[u8; 64],
        fine_difficulty: u8,
    ) -> Result<(XcqaSignature, [u8; 32])> {
        let mut rng = rand::rng();
        let epoch_pk = generate_epoch_pk(block_hash, 0, 0, self.layer_count);

        loop {
            let mut nonce = [0u8; 32];
            rng.fill(&mut nonce);

            let (pk, sk) = xcqa_keygen_privacy(&mut rng, self.layer_count);
            let msg = [block_header, &nonce[..]].concat();
            let sig = xcqa_sign(&msg, &sk, &pk, block_hash);

            let sig_hash = xcqa_crypto::blake3_512(&rkyv::to_bytes::<rkyv::rancor::Error>(&sig).unwrap());

            if check_difficulty(&sig_hash, fine_difficulty) {
                return Ok((sig, nonce));
            }
        }
    }
}
