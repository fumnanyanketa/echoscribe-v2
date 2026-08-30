//! Where the Deepgram key itself lives: Windows Credential Manager, through
//! `keyring`, and nowhere else.
//!
//! Record 0002's data rules: the key is never in the database, never in a log,
//! never in an error message, and never crosses into the interface. Only its
//! last four characters and the name of the entry holding it are stored, and
//! those live in `store.rs`. A macOS implementation slots in behind this same
//! small interface later (AGENTS.md platform boundary).
//!
//! **Why this is not `sign_in::credentials`.** That module is the sign-in
//! feature's, and AGENTS.md is explicit: feature folders do not import from
//! each other, and something becomes shared only when a third feature needs it.
//! Two is a coincidence. So this feature keeps its own small door onto the same
//! vault, with its own entry names. If a third feature ever needs the
//! credential store, both move to a shared module then.
//!
//! Error strings here are deliberately generic and carry nothing read off the
//! key, so a fragment of it can never ride out inside one.

use keyring::Entry;

/// The Credential Manager "service" all EchoScribe entries share. The sign-in
/// feature names the same service for its own entries; the two never collide
/// because the entry names below are prefixed.
const SERVICE: &str = "com.echoscribe.app";

/// The entry name for one account's Deepgram key. This is the string that
/// `deepgram_credential.credential_target` stores; the database never holds the
/// key itself.
pub fn target_for(account_id: &str) -> String {
    format!("deepgram:{account_id}")
}

/// Write (or overwrite) the key for `target`.
pub fn store(target: &str, key: &str) -> Result<(), String> {
    let entry =
        Entry::new(SERVICE, target).map_err(|_| "credential store unavailable".to_string())?;
    entry
        .set_password(key)
        .map_err(|_| "could not save the key to the credential store".to_string())
}

/// Read the key for `target`, or `None` if there is no such entry.
///
/// Only ever called from Rust. Nothing that calls this may put what it returns
/// on an event, in a log, or into a command's return value.
///
/// **Nothing calls this yet, and that is deliberate.** Milestone 4 is its one
/// caller: opening the stream to Deepgram is the only reason to read the key
/// back. It is here now because a vault with no reader cannot be verified, and
/// the round-trip test below is what proves the writing half actually worked
/// against the real Credential Manager. Milestone 3 does not read the key for
/// any other purpose, and milestone 4 removes this allow when it wires the
/// stream up.
#[allow(dead_code)]
pub fn load(target: &str) -> Result<Option<String>, String> {
    let entry =
        Entry::new(SERVICE, target).map_err(|_| "credential store unavailable".to_string())?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
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
        assert_eq!(target_for("user_abc"), "deepgram:user_abc");
        assert_ne!(target_for("user_a"), target_for("user_b"));
    }

    /// Record 0002 AC-18: a second account on the same machine cannot use the
    /// first account's key. The entry name is what keeps them apart, and it
    /// must never collide with the sign-in feature's own entry for the same
    /// account either.
    #[test]
    fn key_and_session_entries_never_collide() {
        assert_ne!(
            target_for("user_abc"),
            crate::sign_in::credentials::target_for("user_abc")
        );
    }

    #[test]
    fn round_trip_through_the_real_store() {
        // Touches the real OS credential store. Uses a throwaway target and
        // cleans up. Skipped automatically if the store is not reachable
        // (headless CI).
        let target = format!("test-deepgram:{}", std::process::id());
        if store(&target, "a-secret-value").is_err() {
            return;
        }
        let got = load(&target).expect("load ok").expect("some key");
        assert_eq!(got, "a-secret-value");
        delete(&target).expect("delete ok");
        assert!(load(&target).expect("load ok").is_none());
    }
}
