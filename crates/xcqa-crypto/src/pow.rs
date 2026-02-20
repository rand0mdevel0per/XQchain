use crate::hash::Hkdf;

// Placeholder for XCQA public key until xcqa crate is available
#[derive(Clone)]
pub struct XcqaPublicKey {
    pub data: Vec<u8>,
}

pub fn generate_epoch_pk(
    prev_block_hash: &[u8; 64],
    block_height: u64,
    difficulty_tier: u8,
    layer_count: usize,
) -> XcqaPublicKey {
    let hkdf = Hkdf::extract(b"XCQA-POW-SALT", prev_block_hash);

    let mut info = Vec::new();
    info.extend_from_slice(b"XCQA-POW-EPOCH-V1");
    info.extend_from_slice(&block_height.to_le_bytes());
    info.push(difficulty_tier);
    info.extend_from_slice(&(layer_count as u64).to_le_bytes());

    let key_material = hkdf.expand(&info, 32 * layer_count);

    XcqaPublicKey { data: key_material }
}

pub fn verify_pow_solution(
    block_header: &[u8],
    xcqa_sig: &[u8],
    xcqa_nonce: &[u8; 32],
    epoch_pk: &XcqaPublicKey,
    fine_difficulty: u8,
) -> bool {
    use crate::hash::blake3_512;

    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(block_header);
    hash_input.extend_from_slice(xcqa_sig);
    hash_input.extend_from_slice(xcqa_nonce);
    hash_input.extend_from_slice(&epoch_pk.data);

    let hash = blake3_512(&hash_input);

    let leading_zeros = hash.iter().take_while(|&&b| b == 0).count();
    leading_zeros >= fine_difficulty as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_epoch_pk() {
        let prev_hash = [1u8; 64];
        let pk = generate_epoch_pk(&prev_hash, 100, 5, 8);
        assert_eq!(pk.data.len(), 32 * 8);
    }

    #[test]
    fn test_verify_pow_solution() {
        let block_header = b"test block header";
        let xcqa_sig = b"test signature";
        let xcqa_nonce = [0u8; 32];
        let epoch_pk = XcqaPublicKey { data: vec![1u8; 256] };

        let result = verify_pow_solution(block_header, xcqa_sig, &xcqa_nonce, &epoch_pk, 0);
        assert!(result);
    }
}
