//! Isolated account persistence. Native handles stay on their owning worker thread.
#[cfg(windows)]
pub mod accounts;
#[cfg(windows)]
pub mod native;
#[cfg(windows)]
pub mod password;
#[cfg(windows)]
pub mod sqlite;
#[cfg(windows)]
pub mod totp;
pub mod varps;
