use thiserror::Error;

#[derive(Error, Debug)]
pub enum PowError {
    #[error("Mining failed: {0}")]
    MiningFailed(String),
    #[error("GPU not available: {0}")]
    GpuNotAvailable(String),
    #[error("Insufficient VRAM: required {required}MB, available {available}MB")]
    InsufficientVram { required: usize, available: usize },
}

pub type Result<T> = std::result::Result<T, PowError>;
