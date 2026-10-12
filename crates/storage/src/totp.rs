use crate::native::Library;
use std::{ffi::c_void, ptr};
pub fn hmac_sha1(secret: &[u8], data: &[u8]) -> Result<[u8; 20], String> {
    let lib = Library::system("bcrypt.dll")?;
    unsafe {
        let open: unsafe extern "system" fn(*mut *mut c_void, *const u16, *const u16, u32) -> i32 =
            std::mem::transmute(lib.symbol("BCryptOpenAlgorithmProvider")?);
        let close: unsafe extern "system" fn(*mut c_void, u32) -> i32 =
            std::mem::transmute(lib.symbol("BCryptCloseAlgorithmProvider")?);
        let create: unsafe extern "system" fn(
            *mut c_void,
            *mut *mut c_void,
            *mut u8,
            u32,
            *const u8,
            u32,
            u32,
        ) -> i32 = std::mem::transmute(lib.symbol("BCryptCreateHash")?);
        let update: unsafe extern "system" fn(*mut c_void, *const u8, u32, u32) -> i32 =
            std::mem::transmute(lib.symbol("BCryptHashData")?);
        let finish: unsafe extern "system" fn(*mut c_void, *mut u8, u32, u32) -> i32 =
            std::mem::transmute(lib.symbol("BCryptFinishHash")?);
        let destroy: unsafe extern "system" fn(*mut c_void) -> i32 =
            std::mem::transmute(lib.symbol("BCryptDestroyHash")?);
        let key_len = u32::try_from(secret.len()).map_err(|_| "HMAC key too long")?;
        let data_len = u32::try_from(data.len()).map_err(|_| "HMAC input too long")?;
        let mut algorithm = ptr::null_mut();
        let name = [83u16, 72, 65, 49, 0];
        let status = open(&mut algorithm, name.as_ptr(), ptr::null(), 8);
        if status < 0 {
            return Err(format!("HMAC provider error {status}"));
        }
        struct Algorithm(
            *mut c_void,
            unsafe extern "system" fn(*mut c_void, u32) -> i32,
        );
        impl Drop for Algorithm {
            fn drop(&mut self) {
                unsafe {
                    (self.1)(self.0, 0);
                }
            }
        }
        let _algorithm = Algorithm(algorithm, close);
        let mut hash = ptr::null_mut();
        let status = create(
            algorithm,
            &mut hash,
            ptr::null_mut(),
            0,
            secret.as_ptr(),
            key_len,
            0,
        );
        if status < 0 {
            return Err(format!("HMAC allocation error {status}"));
        }
        struct Hash(*mut c_void, unsafe extern "system" fn(*mut c_void) -> i32);
        impl Drop for Hash {
            fn drop(&mut self) {
                unsafe {
                    (self.1)(self.0);
                }
            }
        }
        let _hash = Hash(hash, destroy);
        let status = update(hash, data.as_ptr(), data_len, 0);
        if status < 0 {
            return Err(format!("HMAC update error {status}"));
        }
        let mut result = [0; 20];
        let status = finish(hash, result.as_mut_ptr(), 20, 0);
        if status < 0 {
            return Err(format!("HMAC finish error {status}"));
        }
        Ok(result)
    }
}
/// Apache Commons Base32 decoding accepts lowercase and skips nonalphabet characters.
fn decode_base32(value: &[u8]) -> Vec<u8> {
    let mut accumulator = 0u32;
    let mut bits = 0;
    let mut result = Vec::new();
    for &c in value {
        if c == b'=' {
            break;
        }
        let digit = match c.to_ascii_uppercase() {
            b'A'..=b'Z' => c.to_ascii_uppercase() - b'A',
            b'2'..=b'7' => c - b'2' + 26,
            _ => continue,
        };
        accumulator = (accumulator << 5) | digit as u32;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            result.push((accumulator >> bits) as u8);
        }
    }
    result
}
fn generate(mut key: Vec<u8>, unix_seconds: u64) -> Result<String, String> {
    let digest = hmac_sha1(&key, &(unix_seconds / 30).to_be_bytes());
    key.fill(0);
    let digest = digest?;
    let offset = (digest[19] & 15) as usize;
    let binary = u32::from_be_bytes(digest[offset..offset + 4].try_into().unwrap()) & 0x7fffffff;
    Ok(format!("{:06}", binary % 1_000_000))
}
pub fn code(secret_base32: &str, unix_seconds: u64) -> Result<String, String> {
    generate(decode_base32(secret_base32.as_bytes()), unix_seconds)
}
/// GoogleTotp decodes once, then GoogleAuthenticator(byte[]) decodes again.
/// Preserve that stored-secret representation explicitly instead of changing existing accounts.
pub fn reference_code(stored_secret: &str, unix_seconds: u64) -> Result<String, String> {
    let mut first = decode_base32(stored_secret.as_bytes());
    let second = decode_base32(&first);
    first.fill(0);
    generate(second, unix_seconds)
}
pub fn verify(secret_base32: &str, supplied: &str, unix_seconds: u64) -> Result<bool, String> {
    let expected = code(secret_base32, unix_seconds)?;
    if supplied.len() != expected.len() {
        return Ok(false);
    }
    let diff = supplied
        .bytes()
        .zip(expected.bytes())
        .fold(0u8, |v, (a, b)| v | (a ^ b));
    Ok(diff == 0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_totp_vectors_and_exact_time_step() {
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        for (time, expected) in [
            (59, "287082"),
            (1111111109, "081804"),
            (1111111111, "050471"),
            (1234567890, "005924"),
            (2000000000, "279037"),
            (20000000000, "353130"),
        ] {
            assert_eq!(code(secret, time).unwrap(), expected);
            assert!(verify(secret, expected, time).unwrap());
        }
        assert!(!verify(secret, "81804", 1111111109).unwrap());
    }
}
