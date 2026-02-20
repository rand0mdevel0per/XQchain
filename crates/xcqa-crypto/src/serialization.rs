use crate::error::{CryptoError, Result};

pub trait CanonicalSerialize: Sized {
    fn to_canonical_bytes(&self) -> Vec<u8>;
    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self>;
}

impl CanonicalSerialize for u64 {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 8 {
            return Err(CryptoError::InvalidKeyLength { expected: 8, actual: bytes.len() });
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(arr))
    }
}

impl CanonicalSerialize for [u8; 32] {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength { expected: 32, actual: bytes.len() });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(arr)
    }
}

impl CanonicalSerialize for [u8; 64] {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 64 {
            return Err(CryptoError::InvalidKeyLength { expected: 64, actual: bytes.len() });
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(bytes);
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_canonical() {
        let value = 0x0123456789ABCDEFu64;
        let bytes = value.to_canonical_bytes();
        assert_eq!(bytes.len(), 8);
        let decoded = u64::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_array32_canonical() {
        let arr = [42u8; 32];
        let bytes = arr.to_canonical_bytes();
        assert_eq!(bytes.len(), 32);
        let decoded = <[u8; 32]>::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(decoded, arr);
    }
}
