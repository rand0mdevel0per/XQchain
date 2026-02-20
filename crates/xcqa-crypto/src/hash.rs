use blake3::Hasher as Blake3Hasher;
use sha2::{Sha512, Digest};
use hkdf::Hkdf as HkdfImpl;
use sha2::Sha256;

/// BLAKE3-512: 64-byte output via XOF mode
pub fn blake3_512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Blake3Hasher::new();
    hasher.update(data);
    let mut output = [0u8; 64];
    hasher.finalize_xof().fill(&mut output);
    output
}

/// SHA-512: 64-byte output
pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// HKDF-SHA256 for key derivation
pub struct Hkdf {
    prk: [u8; 32],
}

impl Hkdf {
    pub fn extract(salt: &[u8], ikm: &[u8]) -> Self {
        let hkdf = HkdfImpl::<Sha256>::new(Some(salt), ikm);
        let mut prk = [0u8; 32];
        hkdf.expand(&[], &mut prk).expect("HKDF expand failed");
        Self { prk }
    }

    pub fn expand(&self, info: &[u8], length: usize) -> Vec<u8> {
        let hkdf = HkdfImpl::<Sha256>::from_prk(&self.prk).expect("Invalid PRK");
        let mut output = vec![0u8; length];
        hkdf.expand(info, &mut output).expect("HKDF expand failed");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_512() {
        let data = b"test";
        let hash = blake3_512(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sha512() {
        let data = b"test";
        let hash = sha512(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hkdf() {
        let salt = b"salt";
        let ikm = b"input key material";
        let hkdf = Hkdf::extract(salt, ikm);
        let output = hkdf.expand(b"info", 32);
        assert_eq!(output.len(), 32);
    }
}
