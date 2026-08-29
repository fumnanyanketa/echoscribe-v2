//! What screen the interface should draw: the four auth states, and the rules
//! that turn the `session` row plus this launch's refresh result into one of
//! them.
//!
//! The interface never decides this. It calls `get_auth_state` and draws what
//! it is told. No token or Clerk code is ever part of an `AuthState`.

use serde::Serialize;

use super::store::{Account, Store};

/// The single source of truth for which screen is shown.
///
/// Serialised with an internal `state` tag, so the interface branches on
/// `result.state`:
///   `{"state":"signed_out"}`
///   `{"state":"signing_in"}`
///   `{"state":"signed_in","account":{...}}`
///   `{"state":"signed_in_offline","account":{...}}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AuthState {
    /// Nobody is signed in. The sign-in screen is the only thing reachable.
    SignedOut,
    /// A sign-in is underway: the browser is open and we are waiting for the
    /// handback. The interface shows the waiting state with a cancel action.
    SigningIn,
    /// Signed in, and Clerk has confirmed the session this launch.
    SignedIn { account: AccountView },
    /// Signed in from the identity stored last time, but Clerk has not been
    /// reached yet this launch. History and settings work; the interface shows
    /// a "working offline" marker.
    SignedInOffline { account: AccountView },
}

/// The account details the two signed-in states carry. Name, email, initials
/// and the date shown as "signed in since". Never a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountView {
    /// What to show as the person's name: their Clerk name if they gave one,
    /// otherwise the part of their email before the `@`.
    pub display_name: String,
    pub email: String,
    /// One or two letters for the account block, taken from `display_name`.
    pub initials: String,
    /// `account.last_signed_in_at`, a UTC ISO 8601 timestamp. The interface
    /// formats it for display.
    pub signed_in_since: String,
}

impl AccountView {
    pub fn from_account(account: &Account) -> Self {
        let display_name = resolved_display_name(account);
        let initials = initials_from(&display_name);
        Self {
            display_name,
            email: account.email.clone(),
            initials,
            signed_in_since: account.last_signed_in_at.clone(),
        }
    }
}

/// The name to show: the Clerk display name if it is not blank, otherwise the
/// local part of the email (everything before the first `@`).
pub fn resolved_display_name(account: &Account) -> String {
    let name = account.display_name.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    local_part(&account.email)
}

fn local_part(email: &str) -> String {
    match email.split_once('@') {
        Some((local, _)) if !local.is_empty() => local.to_string(),
        _ => email.to_string(),
    }
}

/// One or two uppercase letters from `name`. Splits on spaces and the
/// separators common in email local parts (`.`, `_`, `-`, `+`). Takes the
/// first letter of the first part and, if there is more than one part, the
/// first letter of the last part. Falls back to `?` if there is no letter.
pub fn initials_from(name: &str) -> String {
    let parts: Vec<&str> = name
        .split(|c: char| c.is_whitespace() || matches!(c, '.' | '_' | '-' | '+'))
        .filter(|p| !p.is_empty())
        .collect();

    let first_letter = |s: &str| s.chars().find(|c| c.is_alphanumeric());

    let mut initials = String::new();
    if let Some(first) = parts.first().and_then(|p| first_letter(p)) {
        initials.extend(first.to_uppercase());
    }
    if parts.len() > 1 {
        if let Some(last) = parts.last().and_then(|p| first_letter(p)) {
            initials.extend(last.to_uppercase());
        }
    }

    if initials.is_empty() {
        "?".to_string()
    } else {
        initials
    }
}

/// Turn the stored session and this launch's progress into the state to show.
///
/// `signing_in` is true while the browser handoff is in flight.
/// `refresh_confirmed` is true once Clerk has confirmed the session at least
/// once this launch: a stored account that has not been confirmed yet is
/// `SignedInOffline`, and becomes `SignedIn` after a successful exchange or
/// refresh.
pub fn compute(
    store: &Store,
    signing_in: bool,
    refresh_confirmed: bool,
) -> rusqlite::Result<AuthState> {
    if signing_in {
        return Ok(AuthState::SigningIn);
    }

    let Some(session) = store.read_session()? else {
        return Ok(AuthState::SignedOut);
    };
    if session.account_id.is_empty() {
        return Ok(AuthState::SignedOut);
    }

    let Some(account) = store.read_account(&session.account_id)? else {
        // Session points at an account row that is not there. Treat as signed
        // out rather than guessing at an identity.
        return Ok(AuthState::SignedOut);
    };

    let view = AccountView::from_account(&account);
    if refresh_confirmed {
        Ok(AuthState::SignedIn { account: view })
    } else {
        Ok(AuthState::SignedInOffline { account: view })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(email: &str, display_name: &str) -> Account {
        Account {
            id: "user_1".to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            first_signed_in_at: "2026-08-28T10:00:00Z".to_string(),
            last_signed_in_at: "2026-08-28T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn display_name_prefers_clerk_name() {
        assert_eq!(
            resolved_display_name(&account("sam@example.com", "Sam Rivera")),
            "Sam Rivera"
        );
    }

    #[test]
    fn display_name_falls_back_to_email_local_part() {
        assert_eq!(
            resolved_display_name(&account("sam.rivera@example.com", "")),
            "sam.rivera"
        );
        assert_eq!(
            resolved_display_name(&account("sam.rivera@example.com", "   ")),
            "sam.rivera"
        );
    }

    #[test]
    fn initials_from_a_two_word_name() {
        assert_eq!(initials_from("Sam Rivera"), "SR");
        assert_eq!(initials_from("Jane Q. Smith"), "JS");
    }

    #[test]
    fn initials_from_one_token() {
        assert_eq!(initials_from("sam"), "S");
        assert_eq!(initials_from("madonna"), "M");
    }

    #[test]
    fn initials_from_email_local_part_separators() {
        assert_eq!(initials_from("sam.rivera"), "SR");
        assert_eq!(initials_from("sam_rivera_jones"), "SJ");
        assert_eq!(initials_from("sam+tag"), "ST");
    }

    #[test]
    fn initials_fallback_when_no_letters() {
        assert_eq!(initials_from(""), "?");
        assert_eq!(initials_from("   "), "?");
    }

    #[test]
    fn compute_is_signed_out_with_no_session() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(compute(&store, false, false).unwrap(), AuthState::SignedOut);
    }

    #[test]
    fn compute_is_signed_out_after_clear() {
        let store = Store::open_in_memory().unwrap();
        store
            .upsert_account("u1", "a@example.com", "A", "2026-08-28T10:00:00Z")
            .unwrap();
        store
            .set_session("u1", "target", "2026-08-28T10:00:00Z")
            .unwrap();
        store.clear_session().unwrap();
        assert_eq!(compute(&store, false, false).unwrap(), AuthState::SignedOut);
    }

    #[test]
    fn compute_is_offline_until_refresh_confirmed() {
        let store = Store::open_in_memory().unwrap();
        store
            .upsert_account(
                "u1",
                "sam@example.com",
                "Sam Rivera",
                "2026-08-28T12:00:00Z",
            )
            .unwrap();
        store
            .set_session("u1", "target", "2026-08-28T12:00:00Z")
            .unwrap();

        let offline = compute(&store, false, false).unwrap();
        match offline {
            AuthState::SignedInOffline { account } => {
                assert_eq!(account.display_name, "Sam Rivera");
                assert_eq!(account.initials, "SR");
                assert_eq!(account.email, "sam@example.com");
                assert_eq!(account.signed_in_since, "2026-08-28T12:00:00Z");
            }
            other => panic!("expected offline, got {other:?}"),
        }

        assert!(matches!(
            compute(&store, false, true).unwrap(),
            AuthState::SignedIn { .. }
        ));
    }

    #[test]
    fn compute_is_signing_in_when_flagged() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(compute(&store, true, false).unwrap(), AuthState::SigningIn);
    }

    #[test]
    fn auth_state_serialises_with_a_state_tag() {
        let json = serde_json::to_value(AuthState::SignedOut).unwrap();
        assert_eq!(json, serde_json::json!({ "state": "signed_out" }));

        let view = AccountView {
            display_name: "Sam".to_string(),
            email: "sam@example.com".to_string(),
            initials: "S".to_string(),
            signed_in_since: "2026-08-28T12:00:00Z".to_string(),
        };
        let json = serde_json::to_value(AuthState::SignedIn { account: view }).unwrap();
        assert_eq!(json["state"], "signed_in");
        assert_eq!(json["account"]["initials"], "S");
        assert!(json["account"].get("token").is_none());
    }
}
