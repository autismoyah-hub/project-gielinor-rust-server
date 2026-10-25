use gielinor_protocol::login::{Authentication, Login, Otp};
use gielinor_storage::{
    accounts::{Account, Accounts},
    password::{secure_random, PasswordHasher},
    totp,
};
#[derive(Clone, Copy)]
pub struct Config {
    pub realm: i32,
    pub require_registration: bool,
    pub dev_mode: bool,
    pub ignore_passwords: bool,
}
pub struct Authenticated {
    pub account: Account,
    pub new_account: bool,
    pub authenticator_code: Option<u32>,
}
pub struct Service {
    pub accounts: Accounts,
    pub passwords: PasswordHasher,
    pub config: Config,
}
impl Service {
    /// Execute on the bounded account worker. Returns the reference login status on failure.
    pub fn authenticate(
        &mut self,
        login: Login,
        unix_seconds: u64,
        local_datetime: &str,
    ) -> Result<Authenticated, u8> {
        let Authentication::Password(password) = &login.authentication else {
            return Err(22);
        };
        if password.as_bytes().is_empty() {
            return Err(3);
        }
        let account = self
            .accounts
            .load(&login.username, self.config.realm)
            .map_err(|_| 24u8)?;
        let (account, mut new_account) = if let Some(account) = account {
            (account, false)
        } else {
            if self.config.require_registration {
                return Err(3);
            }
            let hash = self.passwords.hash(password.as_bytes()).map_err(|_| 24u8)?;
            self.accounts
                .create(&login.username, &hash, self.config.realm)
                .map_err(|_| 24u8)?;
            (
                self.accounts
                    .load(&login.username, self.config.realm)
                    .map_err(|_| 24u8)?
                    .ok_or(24u8)?,
                true,
            )
        };
        if !new_account {
            let ignore = self.config.dev_mode && self.config.ignore_passwords;
            if account.twofa_enabled {
                let secret = account.twofa_secret.as_deref().ok_or(57u8)?;
                let failure = match login.otp {
                    Otp::TrustedCode(code) | Otp::UntrustedCode(code) => {
                        let expected =
                            totp::reference_code(secret, unix_seconds).map_err(|_| 24u8)?;
                        if expected != code.to_string() {
                            Some(57)
                        } else {
                            None
                        }
                    }
                    Otp::None => Some(56),
                    Otp::TrustedDevice(device) => {
                        let recent = account
                            .twofa_last_verified
                            .as_deref()
                            .and_then(local_seconds)
                            .zip(local_seconds(local_datetime))
                            .is_some_and(|(last, now)| (now - last) / 86400 < 30);
                        if account.known_device.map(|v| v as u32) != Some(device) || !recent {
                            Some(56)
                        } else {
                            None
                        }
                    }
                };
                if !ignore {
                    if let Some(status) = failure {
                        return Err(status);
                    }
                }
            }
            let verified = self
                .passwords
                .verify(&account.password_hash, password.as_bytes())
                .map_err(|_| 24u8)?;
            if !ignore && !verified {
                return Err(3);
            }
            // A durable first checkpoint records last_login without inventing a
            // logout. It must remain recoverable after a crash before first exit.
            // Never-saved registrations still lack both initialization markers;
            // resume them as the registration they are (default character, new
            // player setup) instead of refusing forever. Duplicate sessions are
            // refused separately: world_id here and world admission (status 5).
            new_account = resumes_registration(&account);
        }
        let authenticator_code = if matches!(login.otp, Otp::None) {
            None
        } else {
            Some(if let Some(device) = account.known_device {
                device as u32
            } else {
                let mut bytes = [0; 4];
                secure_random(&mut bytes).map_err(|_| 24u8)?;
                u32::from_ne_bytes(bytes)
            })
        };
        Ok(Authenticated {
            account,
            new_account,
            authenticator_code,
        })
    }
}
/// A registration whose first login was interrupted before any checkpoint or save.
/// Every save and checkpoint writes last_login in the same transaction as the rest
/// of the character, so nothing else of this character has been persisted either.
fn resumes_registration(account: &Account) -> bool {
    account.last_login.is_none() && account.last_logout.is_none()
}
// LocalDateTime comparisons deliberately do not apply a time-zone/DST offset.
fn local_seconds(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() < 19
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || ![b' ', b'T'].contains(&bytes[10])
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let n = |a, b| std::str::from_utf8(&bytes[a..b]).ok()?.parse::<i64>().ok();
    let mut year = n(0, 4)?;
    let month = n(5, 7)?;
    let day = n(8, 10)?;
    let hour = n(11, 13)?;
    let minute = n(14, 16)?;
    let second = n(17, 19)?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return None;
    }
    year -= if month <= 2 { 1 } else { 0 };
    let era = year.div_euclid(400);
    let y = year - era * 400;
    let m = month + if month > 2 { -3 } else { 9 };
    let days = era * 146097 + y * 365 + y / 4 - y / 100 + (153 * m + 2) / 5 + day - 1;
    Some(days * 86400 + hour * 3600 + minute * 60 + second)
}
pub fn java_name_hash(name: &str) -> u64 {
    let signed = name
        .encode_utf16()
        .fold(0i32, |hash, c| hash.wrapping_mul(31).wrapping_add(c as i32));
    signed as i64 as u64
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_storage::sqlite::Database;
    fn hex(s: &str) -> Vec<u8> {
        let s = s.trim();
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    /// Independently authored PUBLIC TEST ONLY login and deliberately insecure RSA key.
    fn fixture_login() -> Login {
        let key = gielinor_protocol::rsa::PrivateKey::from_be_bytes(
            &hex(include_str!("../tests/fixtures/authentication/rsa-exponent.hex")),
            &hex(include_str!("../tests/fixtures/authentication/rsa-modulus.hex")),
        )
        .unwrap();
        gielinor_protocol::login::decode(
            &hex(include_str!("../tests/fixtures/authentication/login.hex")),
            &key,
        )
        .unwrap()
    }
    /// Always exercises the real public RSA/XTEA decoder, even without Argon2.
    #[test]
    fn public_fixture_login_decodes_without_external_assets() {
        let login = fixture_login();
        assert_eq!(login.header.revision, 239);
        assert_eq!(login.header.subrevision, 1);
        assert_eq!(login.header.server_version, 0);
        assert_eq!(login.header.client_type, 1);
        assert_eq!(login.header.platform_type, 0);
        assert_eq!(login.header.external_authenticator, 0);
        assert_eq!(login.username, "public_auth_190");
        assert_eq!(
            login.seed,
            [0x10203040, 0x50607080, 0x90a0b0c0, 0xd0e0f001]
        );
        assert_eq!(login.server_seed, 0x0102030405060708);
        assert_eq!(login.otp, Otp::None);
        let Authentication::Password(secret) = &login.authentication else {
            panic!("public fixture must use password authentication");
        };
        assert_eq!(secret.as_bytes(), b"public-fixture-only-190");
        assert!(login.low_detail && login.resized);
        assert_eq!((login.width, login.height), (765, 503));
        let machine_id: [u8; 24] = std::array::from_fn(|i| i as u8);
        assert_eq!(login.machine_id, machine_id);
        assert_eq!(login.site_settings, "public-test-only");
        assert_eq!(login.affiliate_id, 0);
        assert_eq!(login.archive_checksums, vec![0x11223344, 0x55667788]);
        assert_eq!(login.platform_blob, vec![0; 55]);
        assert_eq!(login.verification, 0xab);
        assert_eq!(login.client_checksum, 0x12345678);
        let cache_crcs: [u32; 23] = std::array::from_fn(|i| 0xa5000000 | i as u32);
        assert_eq!(login.cache_crcs, cache_crcs);
        assert!(login.trailing_bytes.is_empty());

        let key = gielinor_protocol::rsa::PrivateKey::from_be_bytes(
            &hex(include_str!("../tests/fixtures/authentication/rsa-exponent.hex")),
            &hex(include_str!("../tests/fixtures/authentication/rsa-modulus.hex")),
        )
        .unwrap();
        let payload = hex(include_str!("../tests/fixtures/authentication/login.hex"));
        let mut wrong_revision = payload.clone();
        wrong_revision[3] = 238;
        assert!(gielinor_protocol::login::decode(&wrong_revision, &key).is_err());
        let mut empty_rsa = payload.clone();
        empty_rsa[15..17].copy_from_slice(&[0, 0]);
        assert!(gielinor_protocol::login::decode(&empty_rsa, &key).is_err());
        assert!(gielinor_protocol::login::decode(&payload[..15], &key).is_err());
    }
    /// argon2.dll is an external asset, found like the launchers do.
    fn hasher() -> Option<PasswordHasher> {
        let root = std::env::var_os("OSRS_ASSET_ROOT")?;
        let path = std::path::Path::new(&root).join("rust-server/work/native/argon2.dll");
        path.is_file()
            .then(|| PasswordHasher::load(&path.canonicalize().unwrap()).unwrap())
    }
    #[test]
    fn only_never_checkpointed_registrations_resume() {
        let mut accounts = Accounts::from_database(Database::memory().unwrap());
        accounts.initialize().unwrap();
        accounts.create("interrupted", "test-hash", 1).unwrap();
        let mut account = accounts.load("interrupted", 1).unwrap().unwrap();
        assert!(resumes_registration(&account));
        account.last_login = Some("2026-10-24 06:02:55".into());
        assert!(!resumes_registration(&account));
        account.last_logout = Some("2026-10-24 06:10:00".into());
        assert!(!resumes_registration(&account));
        account.last_login = None;
        assert!(!resumes_registration(&account));
    }
    /// Issue #13: a session killed during its first login (no checkpoint) used to
    /// leave the account refusing every later login with status 13.
    #[test]
    fn interrupted_first_login_resumes_after_credentials() {
        let Some(passwords) = hasher() else {
            eprintln!("SKIPPED interrupted_first_login_resumes_after_credentials: no argon2.dll under $OSRS_ASSET_ROOT/rust-server/work/native (external asset)");
            return;
        };
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("gielinor-interrupted-login-{stamp}.sqlite"));
        let mut accounts = Accounts::from_database(Database::open(&path, false).unwrap());
        accounts.initialize().unwrap();
        // A NULL/NULL registration whose stored hash is for another password.
        accounts
            .create("stranger", &passwords.hash(b"some other password").unwrap(), 1)
            .unwrap();
        let mut service = Service {
            accounts,
            passwords,
            config: Config {
                realm: 1,
                require_registration: false,
                dev_mode: false,
                ignore_passwords: false,
            },
        };
        let now = "2026-10-24 06:02:55";
        let registered = service.authenticate(fixture_login(), 0, now).ok().unwrap();
        assert!(registered.new_account);
        // The session dies before the first checkpoint: both markers stay NULL.
        assert!(registered.account.last_login.is_none() && registered.account.last_logout.is_none());
        let resumed = service.authenticate(fixture_login(), 0, now).ok().unwrap();
        assert!(resumed.new_account);
        assert_eq!(resumed.account.character_id, registered.account.character_id);
        // Resuming still requires the account's own credentials.
        let mut stranger = fixture_login();
        stranger.username = "stranger".into();
        assert_eq!(service.authenticate(stranger, 0, now).err(), Some(3));
        // After the first checkpoint the account is an ordinary returning one.
        let raw = Database::open(&path, false).unwrap();
        raw.execute(
            "UPDATE characters SET last_login='2026-10-24 06:03:00' WHERE id=(SELECT c.id FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username='public_auth_190')",
            &[],
        )
        .unwrap();
        drop(raw);
        let returning = service.authenticate(fixture_login(), 0, now).ok().unwrap();
        assert!(!returning.new_account);
        drop(service);
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }
    #[test]
    fn trusted_device_uses_calendar_days() {
        assert_eq!(
            local_seconds("2026-10-31 12:00:00").unwrap()
                - local_seconds("2026-10-01 12:00:00").unwrap(),
            30 * 86400
        );
        assert_eq!(
            local_seconds("2024-03-01T00:00:00").unwrap()
                - local_seconds("2024-02-28 00:00:00").unwrap(),
            2 * 86400
        );
        assert_eq!(java_name_hash("abc"), 96354);
    }
}
