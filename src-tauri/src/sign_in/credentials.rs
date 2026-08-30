//! Where the sign-in feature reads and writes the OS credential store.
//!
//! This used to say it was the one place in EchoScribe that did. It is not, as
//! of milestone 3 of record 0002: the dictate feature keeps its own door onto
//! the same vault in `dictate/key_vault.rs`, for the person's Deepgram key.
//! AGENTS.md is why they are separate rather than shared. Feature folders do
//! not import from each other, and something becomes shared only when a third
//! feature needs it. Two is a coincidence. The entry names are prefixed, so the
//! two never collide.
//!
//! Windows Credential Manager today, through `keyring`. A macOS implementation
//! slots in behind this same small interface later (AGENTS.md platform
//! boundary). The session tokens live only here: never in the database, never
//! in a log, never in a plain file. Error strings from this module are
//! deliberately generic so a token fragment can never ride out inside one.

use keyring::Entry;
use serde::{Deserialize, Serialize};

/// The Credential Manager "service" all EchoScribe entries share.
const SERVICE: &str = "com.echoscribe.app";

/// What is kept for a signed-in account: the long-lived refresh token, the
/// current access token, and when that access token stops working.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix seconds. When the access token expires.
    pub access_expires_at: i64,
}

/// The entry name for one account's session tokens. This is the string that
/// `session.credential_target` stores; the database never holds the tokens
/// themselves.
pub fn target_for(account_id: &str) -> String {
    format!("session:{account_id}")
}

/// Write (or overwrite) the tokens for `target`.
pub fn store(target: &str, tokens: &SessionTokens) -> Result<(), String> {
    let entry =
        Entry::new(SERVICE, target).map_err(|_| "credential store unavailable".to_string())?;
    let blob = serde_json::to_string(tokens).map_err(|_| "could not encode session".to_string())?;
    entry
        .set_password(&blob)
        .map_err(|_| "could not save session to the credential store".to_string())
}

/// Read the tokens for `target`, or `None` if there is no such entry.
pub fn load(target: &str) -> Result<Option<SessionTokens>, String> {
    let entry =
        Entry::new(SERVICE, target).map_err(|_| "credential store unavailable".to_string())?;
    match entry.get_password() {
        Ok(blob) => serde_json::from_str(&blob)
            .map(Some)
            .map_err(|_| "stored session is unreadable".to_string()),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("could not read the credential store".to_string()),
    }
}

/// Remove the entry for `target`. Succeeds if it was already gone.
pub fn delete(target: &str) -> Result<(), String> {
    let entry =
        Entry::new(SERVICE, target).map_err(|_| "credential store unavailable".to_string())?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("could not clear the credential store".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_name_is_per_account() {
        assert_eq!(target_for("user_abc"), "session:user_abc");
        assert_ne!(target_for("user_a"), target_for("user_b"));
    }

    #[test]
    fn round_trip_through_the_real_store() {
        // Touches the real OS credential store. Uses a throwaway target and
        // cleans up. Skipped automatically if the store is not reachable
        // (headless CI).
        let target = format!("test:{}", std::process::id());
        let tokens = SessionTokens {
            access_token: "acc".to_string(),
            refresh_token: "ref".to_string(),
            access_expires_at: 1_787_923_245,
        };
        if store(&target, &tokens).is_err() {
            return;
        }
        let got = load(&target).expect("load ok").expect("some tokens");
        assert_eq!(got.refresh_token, "ref");
        assert_eq!(got.access_expires_at, 1_787_923_245);
        delete(&target).expect("delete ok");
        assert!(load(&target).expect("load ok").is_none());
    }
}
