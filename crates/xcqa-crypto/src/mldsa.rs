use ml_dsa::{MlDsa65, SigningKey, VerifyingKey, Signature, Seed};
use ml_dsa::signature::{Signer, Verifier};
use rand::{Rng, CryptoRng, RngExt};
use zeroize::Zeroize;
use serde::{Serialize, Deserialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct MlDsa65PublicKey(#[serde(with = "serde_big_array::BigArray")] pub [u8; 1952]);

#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MlDsa65PrivateKey([u8; 32]);

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct MlDsa65Signature(#[serde(with = "serde_big_array::BigArray")] pub [u8; 3309]);

pub fn mldsa_keygen<R: Rng + CryptoRng>(rng: &mut R) -> (MlDsa65PublicKey, MlDsa65PrivateKey) {
    let mut seed_bytes = [0u8; 32];
    rng.fill(&mut seed_bytes);

    let seed: Seed = seed_bytes.into();
    let signing_key = SigningKey::<MlDsa65>::from_seed(&seed);
    let verifying_key = signing_key.verifying_key();

    let pk_encoded = verifying_key.encode();
    let mut pk = [0u8; 1952];
    pk.copy_from_slice(pk_encoded.as_ref());

    (MlDsa65PublicKey(pk), MlDsa65PrivateKey(seed_bytes))
}

pub fn mldsa_sign(message: &[u8], sk: &MlDsa65PrivateKey) -> MlDsa65Signature {
    let seed: Seed = sk.0.into();
    let signing_key = SigningKey::<MlDsa65>::from_seed(&seed);
    let sig_encoded = signing_key.sign(message).encode();
    let mut sig = [0u8; 3309];
    sig.copy_from_slice(sig_encoded.as_ref());
    MlDsa65Signature(sig)
}

pub fn mldsa_verify(message: &[u8], signature: &MlDsa65Signature, pk: &MlDsa65PublicKey) -> bool {
    let vk = VerifyingKey::<MlDsa65>::decode(&pk.0.into());
    let sig = match Signature::<MlDsa65>::decode(&signature.0.into()) {
        Some(s) => s,
        None => return false,
    };
    vk.verify(message, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mldsa_sign_verify() {
        let mut rng = rand::rng();
        let (pk, sk) = mldsa_keygen(&mut rng);
        let message = b"test message";
        let sig = mldsa_sign(message, &sk);
        assert!(mldsa_verify(message, &sig, &pk));
    }

    #[test]
    fn test_mldsa_invalid_signature() {
        let mut rng = rand::rng();
        let (pk, sk) = mldsa_keygen(&mut rng);
        let message = b"test message";
        let sig = mldsa_sign(message, &sk);
        let wrong_message = b"wrong message";
        assert!(!mldsa_verify(wrong_message, &sig, &pk));
    }
}
