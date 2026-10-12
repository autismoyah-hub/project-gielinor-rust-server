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
        let (account, new_account) = if let Some(account) = account {
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
            if account.last_logout.is_none() {
                return Err(13);
            }
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
