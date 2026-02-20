use serde::{Serialize, Deserialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Clone)]
pub struct XcqaPublicKeyWrapped {
    pub layers: usize,
    pub(crate) inner: xcqa::PublicKey,
}

#[derive(Clone)]
pub struct XcqaPrivateKeyWrapped {
    pub layers: usize,
    pub(crate) inner: xcqa::PrivateKey,
}

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct XcqaSignature {
    pub commitment: Vec<u8>,
    pub response: Vec<u8>,
}

pub fn xcqa_keygen_privacy<R>(
    _rng: &mut R,
    layers: usize,
) -> (XcqaPublicKeyWrapped, XcqaPrivateKeyWrapped)
where
    R: rand::Rng + rand::CryptoRng,
{
    let (pk, sk) = xcqa::keygen();
    (
        XcqaPublicKeyWrapped { layers, inner: pk },
        XcqaPrivateKeyWrapped { layers, inner: sk },
    )
}

pub fn xcqa_sign(
    msg: &[u8],
    sk: &XcqaPrivateKeyWrapped,
    pk: &XcqaPublicKeyWrapped,
    block_hash: &[u8; 64],
) -> XcqaSignature {
    let sig = xcqa::sign_with_context(msg, &sk.inner, &pk.inner, block_hash);
    XcqaSignature {
        commitment: sig.commitment.clone(),
        response: sig.response.clone(),
    }
}

pub fn xcqa_verify(
    msg: &[u8],
    sig: &XcqaSignature,
    pk: &XcqaPublicKeyWrapped,
    block_hash: &[u8; 64],
) -> bool {
    let xcqa_sig = xcqa::Signature {
        commitment: sig.commitment.clone(),
        response: sig.response.clone(),
    };
    xcqa::verify_with_context(msg, &xcqa_sig, &pk.inner, block_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcqa_keygen_privacy() {
        let mut rng = rand::rng();
        let (pk, sk) = xcqa_keygen_privacy(&mut rng, 7);
        assert_eq!(pk.layers, 7);
        assert_eq!(sk.layers, 7);
    }

    #[test]
    fn test_xcqa_sign_verify() {
        let mut rng = rand::rng();
        let (pk, sk) = xcqa_keygen_privacy(&mut rng, 8);
        let msg = b"test message";
        let block_hash = [1u8; 64];

        let sig = xcqa_sign(msg, &sk, &pk, &block_hash);
        assert!(xcqa_verify(msg, &sig, &pk, &block_hash));
    }

    #[test]
    fn test_xcqa_verify_wrong_message() {
        let mut rng = rand::rng();
        let (pk, sk) = xcqa_keygen_privacy(&mut rng, 8);
        let msg = b"test message";
        let wrong_msg = b"wrong message";
        let block_hash = [1u8; 64];

        let sig = xcqa_sign(msg, &sk, &pk, &block_hash);
        assert!(!xcqa_verify(wrong_msg, &sig, &pk, &block_hash));
    }
}
