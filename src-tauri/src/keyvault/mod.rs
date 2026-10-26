//! OS keychain integration (anticipates P3-T06).
//!
//! Wraps the `keyring` crate so we can store and retrieve secrets per host id.
//! The service name is fixed (`SERVICE`) and the account name is the host UUID,
//! which keeps everything namespaced and easy to clean up.
//!
//! Errors from the underlying crate are flattened to `anyhow::Error` because
//! the frontend doesn't need to distinguish "not found" from "backend error"
//! at this stage — the caller just falls back to prompting the user.

use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "io.github.moha528.lynk";

fn entry(account: &str) -> Result<Entry> {
    Entry::new(SERVICE, account).with_context(|| format!("keyring entry {account}"))
}

/// Persist `secret` for `account` in the OS keychain.
pub fn set_secret(account: &str, secret: &str) -> Result<()> {
    entry(account)?
        .set_password(secret)
        .with_context(|| format!("store secret for {account}"))
}

/// Return the stored secret for `account`, or `None` if no entry exists.
pub fn get_secret(account: &str) -> Result<Option<String>> {
    match entry(account)?.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::Error::new(e).context(format!("read secret for {account}"))),
    }
}

/// Remove the secret for `account`. Returns `true` if an entry was deleted,
/// `false` if no entry existed.
pub fn delete_secret(account: &str) -> Result<bool> {
    match entry(account)?.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(anyhow::Error::new(e).context(format!("delete secret for {account}"))),
    }
}

/// Route every keychain call of the test binary to keyring's in-memory mock
/// store. Tests then never write to the developer's real keychain, and they
/// run on machines without one (GitHub's Ubuntu runners have no Secret
/// Service). Must be called before the test creates its first entry.
#[cfg(test)]
pub(crate) fn use_mock_store() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    });
}
