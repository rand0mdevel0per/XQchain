use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Invalid proof")]
    InvalidProof,
}

pub type Result<T> = std::result::Result<T, CryptoError>;
