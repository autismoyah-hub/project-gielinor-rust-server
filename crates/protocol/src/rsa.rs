use crate::buffer::DecodeError;
use num_bigint::BigUint;

/// Raw RSA is the legacy game protocol primitive, not a general-purpose encryption API.
/// Provision isolated keys separately. Never include private material in Debug/log output.
pub struct PrivateKey {
    exponent: BigUint,
    modulus: BigUint,
}
impl PrivateKey {
    pub fn from_be_bytes(exponent: &[u8], modulus: &[u8]) -> Result<Self, DecodeError> {
        let exponent = BigUint::from_bytes_be(exponent);
        let modulus = BigUint::from_bytes_be(modulus);
        if exponent.bits() == 0 || modulus.bits() < 16 || modulus.bits() > 4096 {
            return Err(DecodeError::InvalidValue("RSA key"));
        }
        Ok(Self { exponent, modulus })
    }
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, DecodeError> {
        if encrypted.is_empty() || encrypted.len() > self.modulus.bits().div_ceil(8) as usize + 1 {
            return Err(DecodeError::InvalidValue("RSA block length"));
        }
        let input = BigUint::from_bytes_be(encrypted);
        if input >= self.modulus {
            return Err(DecodeError::InvalidValue("RSA ciphertext"));
        }
        Ok(input.modpow(&self.exponent, &self.modulus).to_bytes_be())
    }
}
