use crate::native::Library;
use std::{
    ffi::{c_char, c_void, CStr, CString},
    path::Path,
};
type Verify = unsafe extern "C" fn(*const c_char, *const c_void, usize) -> i32;
type Hash = unsafe extern "C" fn(
    u32,
    u32,
    u32,
    *const c_void,
    usize,
    *const c_void,
    usize,
    usize,
    *mut c_char,
    usize,
) -> i32;
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Native(String),
    InvalidEncoding,
    CostLimit,
}
/// The Kotlin factory's Argon2i parameters: 20 iterations, 65536 KiB, one lane.
/// Run on a bounded account worker, never on the game tick thread.
pub struct PasswordHasher {
    hash_i: Hash,
    verify_i: Verify,
    verify_id: Verify,
    _library: Library,
}
impl PasswordHasher {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let library = Library::absolute(path).map_err(Error::Native)?;
        unsafe {
            Ok(Self {
                hash_i: std::mem::transmute(
                    library
                        .symbol("argon2i_hash_encoded")
                        .map_err(Error::Native)?,
                ),
                verify_i: std::mem::transmute(
                    library.symbol("argon2i_verify").map_err(Error::Native)?,
                ),
                verify_id: std::mem::transmute(
                    library.symbol("argon2id_verify").map_err(Error::Native)?,
                ),
                _library: library,
            })
        }
    }
    pub fn hash(&self, password: &[u8]) -> Result<String, Error> {
        let mut salt = [0u8; 16];
        secure_random(&mut salt)?;
        self.hash_with_salt(password, &salt)
    }
    fn hash_with_salt(&self, password: &[u8], salt: &[u8; 16]) -> Result<String, Error> {
        let mut encoded = [0u8; 512];
        let code = unsafe {
            (self.hash_i)(
                20,
                65536,
                1,
                password.as_ptr().cast(),
                password.len(),
                salt.as_ptr().cast(),
                salt.len(),
                32,
                encoded.as_mut_ptr().cast(),
                encoded.len(),
            )
        };
        if code != 0 {
            return Err(Error::Native(format!("Argon2 error {code}")));
        }
        Ok(unsafe { CStr::from_ptr(encoded.as_ptr().cast()) }
            .to_str()
            .map_err(|_| Error::InvalidEncoding)?
            .into())
    }
    pub fn verify(&self, encoded: &str, password: &[u8]) -> Result<bool, Error> {
        if encoded.len() > 512 {
            return Err(Error::InvalidEncoding);
        }
        let parts: Vec<_> = encoded.split('$').collect();
        if parts.len() != 6 || parts[0] != "" || parts[2] != "v=19" {
            return Err(Error::InvalidEncoding);
        }
        let verify = match parts[1] {
            "argon2i" => self.verify_i,
            "argon2id" => self.verify_id,
            _ => return Err(Error::InvalidEncoding),
        };
        let mut memory = None;
        let mut iterations = None;
        let mut parallel = None;
        for field in parts[3].split(',') {
            let (key, value) = field.split_once('=').ok_or(Error::InvalidEncoding)?;
            let value = value.parse::<u32>().map_err(|_| Error::InvalidEncoding)?;
            let slot = match key {
                "m" => &mut memory,
                "t" => &mut iterations,
                "p" => &mut parallel,
                _ => return Err(Error::InvalidEncoding),
            };
            if slot.replace(value).is_some() {
                return Err(Error::InvalidEncoding);
            }
        }
        if !memory.is_some_and(|v| (8..=131072).contains(&v))
            || !iterations.is_some_and(|v| (1..=32).contains(&v))
            || !parallel.is_some_and(|v| (1..=2).contains(&v))
        {
            return Err(Error::CostLimit);
        }
        let encoded = CString::new(encoded).map_err(|_| Error::InvalidEncoding)?;
        let code = unsafe { verify(encoded.as_ptr(), password.as_ptr().cast(), password.len()) };
        match code {
            0 => Ok(true),
            -35 => Ok(false),
            _ => Err(Error::Native(format!("Argon2 verification error {code}"))),
        }
    }
}
pub fn secure_random(output: &mut [u8]) -> Result<(), Error> {
    let library = Library::system("bcrypt.dll").map_err(Error::Native)?;
    let generate: unsafe extern "system" fn(*mut c_void, *mut u8, u32, u32) -> i32 =
        unsafe { std::mem::transmute(library.symbol("BCryptGenRandom").map_err(Error::Native)?) };
    let length = u32::try_from(output.len()).map_err(|_| Error::CostLimit)?;
    let status = unsafe { generate(std::ptr::null_mut(), output.as_mut_ptr(), length, 2) };
    if status < 0 {
        return Err(Error::Native(format!("OS entropy error {status}")));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    /// The external Argon2 library, found like the launchers do: beneath `OSRS_ASSET_ROOT`
    /// (`scripts/start-server.ps1`), else beside a standalone rust-server checkout.
    fn reference_argon2() -> Option<std::path::PathBuf> {
        let relative = Path::new("rust-server/work/native/argon2.dll");
        std::env::var_os("OSRS_ASSET_ROOT")
            .map(|root| Path::new(&root).join(relative))
            .into_iter()
            .chain([Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/native/argon2.dll")])
            .find(|path| path.is_file())
    }
    #[test]
    fn cached_reference_argon2_verifies_and_rejects_wrong_password() {
        let Some(path) = reference_argon2() else {
            eprintln!(
                "SKIPPED cached_reference_argon2_verifies_and_rejects_wrong_password: \
                 no argon2.dll under $OSRS_ASSET_ROOT/rust-server/work/native or the checkout's \
                 work/native (external asset, not shipped)"
            );
            return;
        };
        let hasher = PasswordHasher::load(&path.canonicalize().unwrap()).unwrap();
        let hash = hasher
            .hash_with_salt(b"parity-only-password", &[23; 16])
            .unwrap();
        assert!(hash.starts_with("$argon2i$v=19$m=65536,t=20,p=1$"));
        assert!(hasher.verify(&hash, b"parity-only-password").unwrap());
        assert!(!hasher.verify(&hash, b"wrong").unwrap());
        assert_eq!(
            hasher.verify("$argon2i$v=19$m=999999999,t=20,p=1$a$b", b"x"),
            Err(Error::CostLimit)
        );
    }
}
