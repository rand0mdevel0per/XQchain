#[cfg(feature = "gpu")]
use cudarc::driver::{CudaDevice, CudaStream};
use xcqa_crypto::{generate_epoch_pk, xcqa_keygen_privacy, xcqa_sign, XcqaSignature};
use crate::difficulty::check_difficulty;
use crate::error::{PowError, Result};
use rand::{Rng, RngExt};

pub struct GpuSolver {
    layer_count: usize,
    #[cfg(feature = "gpu")]
    device: Option<CudaDevice>,
}

impl GpuSolver {
    pub fn new(layer_count: usize, min_vram_mb: usize) -> Result<Self> {
        #[cfg(feature = "gpu")]
        {
            match CudaDevice::new(0) {
                Ok(device) => {
                    let total_mem = device.total_memory().map_err(|e|
                        PowError::GpuNotAvailable(format!("Failed to query VRAM: {}", e))
                    )?;
                    let available_mb = total_mem / (1024 * 1024);

                    if available_mb < min_vram_mb {
                        return Err(PowError::InsufficientVram {
                            required: min_vram_mb,
                            available: available_mb,
                        });
                    }

                    Ok(Self { layer_count, device: Some(device) })
                }
                Err(e) => Err(PowError::GpuNotAvailable(format!("CUDA device not found: {}", e))),
            }
        }

        #[cfg(not(feature = "gpu"))]
        Err(PowError::GpuNotAvailable("GPU feature not enabled".into()))
    }

    pub fn mine(
        &self,
        block_header: &[u8],
        block_hash: &[u8; 64],
        fine_difficulty: u8,
    ) -> Result<(XcqaSignature, [u8; 32])> {
        let mut rng = rand::rng();

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
