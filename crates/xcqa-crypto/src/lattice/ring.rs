use super::params::{Q, N};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Clone, Debug, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct RingElement {
    pub coeffs: [u32; N],
}

impl Serialize for RingElement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.coeffs.as_slice().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RingElement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let vec = Vec::<u32>::deserialize(deserializer)?;
        let mut coeffs = [0u32; N];
        coeffs.copy_from_slice(&vec);
        Ok(Self { coeffs })
    }
}

impl RingElement {
    pub fn zero() -> Self {
        Self { coeffs: [0; N] }
    }

    pub fn add(&self, other: &Self) -> Self {
        let mut result = [0u32; N];
        for i in 0..N {
            result[i] = (self.coeffs[i] + other.coeffs[i]) % Q;
        }
        Self { coeffs: result }
    }

    pub fn sub(&self, other: &Self) -> Self {
        let mut result = [0u32; N];
        for i in 0..N {
            result[i] = (self.coeffs[i] + Q - other.coeffs[i]) % Q;
        }
        Self { coeffs: result }
    }

    pub fn scalar_mul(&self, scalar: u64) -> Self {
        let mut result = [0u32; N];
        let s = (scalar % Q as u64) as u32;
        for i in 0..N {
            result[i] = ((self.coeffs[i] as u64 * s as u64) % Q as u64) as u32;
        }
        Self { coeffs: result }
    }

    pub fn mul(&self, other: &Self) -> Self {
        let mut result = [0i64; 2 * N];

        // Schoolbook multiplication
        for i in 0..N {
            for j in 0..N {
                result[i + j] += (self.coeffs[i] as i64) * (other.coeffs[j] as i64);
            }
        }

        // Reduce modulo (X^N + 1): X^N ≡ -1
        let mut coeffs = [0u32; N];
        for i in 0..N {
            let mut c = result[i];
            if i + N < 2 * N {
                c -= result[i + N];
            }
            coeffs[i] = ((c % Q as i64 + Q as i64) % Q as i64) as u32;
        }

        Self { coeffs }
    }
}

#[derive(Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct RingVector {
    pub elements: Vec<RingElement>,
}

impl RingVector {
    pub fn zero(size: usize) -> Self {
        Self {
            elements: vec![RingElement::zero(); size],
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.elements.len(), other.elements.len());
        Self {
            elements: self.elements.iter()
                .zip(&other.elements)
                .map(|(a, b)| a.add(b))
                .collect(),
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.elements.len(), other.elements.len());
        Self {
            elements: self.elements.iter()
                .zip(&other.elements)
                .map(|(a, b)| a.sub(b))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_mul_basic() {
        let mut a = RingElement::zero();
        let mut b = RingElement::zero();
        a.coeffs[0] = 5;
        b.coeffs[0] = 3;
        let c = a.mul(&b);
        assert_eq!(c.coeffs[0], 15);
    }

    #[test]
    fn test_ring_mul_reduction() {
        let mut a = RingElement::zero();
        let mut b = RingElement::zero();
        a.coeffs[255] = 2;
        b.coeffs[1] = 3;
        let c = a.mul(&b);
        // X^255 * X^1 = X^256 = -1 (mod X^256+1)
        assert_eq!(c.coeffs[0], Q - 6);
    }
}
