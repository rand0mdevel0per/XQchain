use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(#[from] xcqa_crypto::CryptoError),
}

pub type Result<T> = std::result::Result<T, CoreError>;
