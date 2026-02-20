use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

pub type Result<T> = std::result::Result<T, NetError>;
