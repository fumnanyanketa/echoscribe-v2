//! Clerk OAuth application configuration for EchoScribe.
//!
//! None of this is secret. EchoScribe is a public OAuth client using PKCE, so
//! there is no client secret anywhere in the app (docs/decisions/0003-sign-in.md).
//! Change these only when the Clerk application changes.

/// The Clerk instance domain, also called the Frontend API domain. This is the
/// production instance, on the `echoscribe.exentrik.co` subdomain.
pub const CLERK_DOMAIN: &str = "clerk.echoscribe.exentrik.co";

/// The production OAuth application's client id. Public client, no secret.
pub const OAUTH_CLIENT_ID: &str = "e9UCEAbRxiyaIiCA";

/// Requested scopes. `offline_access` is what makes Clerk return a refresh
/// token, which is what keeps a person signed in across restarts.
pub const SCOPES: &[&str] = &["openid", "profile", "email", "offline_access"];

/// Path half of the loopback redirect. The full redirect URI at runtime is
/// `http://127.0.0.1:<port>/callback`, with a fresh free port each sign-in.
/// Clerk has `http://127.0.0.1/callback` on its allow list and accepts the
/// dynamic port for a loopback address.
pub const REDIRECT_PATH: &str = "/callback";

/// How long to wait for the browser handback before giving up
/// (docs/decisions/0003-sign-in.md AC-12). Fixed here, not a setting.
pub const SIGN_IN_TIMEOUT_SECS: u64 = 300;

/// How long a silent refresh or a sign-out revocation may take before it counts
/// as Clerk not answering. Nothing on screen waits on either one, so this only
/// stops a stalled connection from holding the upkeep thread forever.
pub const CLERK_CALL_TIMEOUT_SECS: u64 = 15;

/// How long the "can we reach Clerk at all" check may take before a sign-in is
/// stopped before it starts (docs/decisions/0003-sign-in.md AC-15). Long enough
/// for a slow phone hotspot, short enough that a dead connection is not a
/// stare. Fixed here, not a setting.
pub const REACH_CHECK_TIMEOUT_SECS: u64 = 5;

/// How often the upkeep thread looks at the session.
pub const UPKEEP_INTERVAL_SECS: u64 = 60;

/// Renew the access token once it has less than this long left to live.
pub const RENEW_MARGIN_SECS: i64 = 300;

pub fn authorize_url() -> String {
    format!("https://{CLERK_DOMAIN}/oauth/authorize")
}

pub fn token_url() -> String {
    format!("https://{CLERK_DOMAIN}/oauth/token")
}

pub fn userinfo_url() -> String {
    format!("https://{CLERK_DOMAIN}/oauth/userinfo")
}

/// The public OpenID discovery document. Needs no token and carries no query,
/// so asking for it says nothing about who is using the app. Fetched only to
/// learn whether Clerk answers at all.
pub fn discovery_url() -> String {
    format!("https://{CLERK_DOMAIN}/.well-known/openid-configuration")
}

/// Where a refresh token is revoked at sign-out. Read from this instance's
/// discovery document as `revocation_endpoint`.
pub fn revoke_url() -> String {
    format!("https://{CLERK_DOMAIN}/oauth/token/revoke")
}
