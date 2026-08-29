//! Talking to Clerk's OAuth endpoints: exchanging the authorization code for
//! tokens, and reading the account from userinfo. Hand wired against the
//! endpoints, because Clerk has no desktop SDK
//! (docs/decisions/0003-sign-in.md).
//!
//! Every error out of this module is one of the named, safe-to-show reasons.
//! Diagnostics printed here name the failing step and standard OAuth error
//! codes only. They never print a token, an authorization code, an email, a
//! name, or a raw response body.

use std::time::Duration;

use oauth2::basic::BasicClient;
use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, PkceCodeVerifier, RedirectUrl, RefreshToken,
    RequestTokenError, TokenResponse, TokenUrl,
};
use serde::Deserialize;

use super::clock::now_unix;
use super::config;
use super::credentials::SessionTokens;

/// A sign-in that did not complete, named so the screen can show one wording
/// per reason (docs/decisions/0003-sign-in.md "Interface surface").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInError {
    /// The person closed or abandoned the browser.
    BrowserClosed,
    /// No callback arrived within the timeout.
    TimedOut,
    /// The returned state or PKCE did not match this attempt.
    SecurityCheckFailed,
    /// Clerk refused the sign-in.
    ClerkRejected,
    /// Clerk could not be reached at all.
    CannotReachClerk,
}

impl SignInError {
    /// The stable code the interface maps to a sentence.
    pub fn code(self) -> &'static str {
        match self {
            Self::BrowserClosed => "browser_closed",
            Self::TimedOut => "timed_out",
            Self::SecurityCheckFailed => "security_check_failed",
            Self::ClerkRejected => "clerk_rejected",
            Self::CannotReachClerk => "cannot_reach_clerk",
        }
    }
}

/// Why a silent refresh produced no new tokens. The two cases are treated
/// completely differently, which is the whole point of separating them
/// (docs/decisions/0003-sign-in.md AC-14 and AC-16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshError {
    /// Clerk answered and refused the refresh token. The session is over:
    /// signed out elsewhere, revoked, or expired. The person is signed out.
    Refused,
    /// Clerk could not be reached at all. Says nothing about the session, so
    /// the session is left exactly as it was and the app works offline.
    Unreachable,
}

/// Who the person is, from Clerk's userinfo. Never carries a token.
pub struct AccountInfo {
    pub id: String,
    pub email: String,
    pub name: String,
}

/// Exchange the authorization code (plus the PKCE verifier from this attempt)
/// for tokens. No client secret is sent: EchoScribe is a public client, so the
/// `client_id` goes in the request body (`AuthType::RequestBody`).
pub fn exchange_code(
    code: &str,
    pkce_verifier: &str,
    redirect_uri: &str,
) -> Result<SessionTokens, SignInError> {
    let client = BasicClient::new(ClientId::new(config::OAUTH_CLIENT_ID.to_string()))
        .set_auth_type(AuthType::RequestBody)
        .set_auth_uri(
            AuthUrl::new(config::authorize_url()).map_err(|_| SignInError::ClerkRejected)?,
        )
        .set_token_uri(TokenUrl::new(config::token_url()).map_err(|_| SignInError::ClerkRejected)?)
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .map_err(|_| SignInError::SecurityCheckFailed)?,
        );

    let http = http_client(None).map_err(|_| SignInError::CannotReachClerk)?;

    let token = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
        .request(&http)
        .map_err(classify_token_error)?;

    let refresh_token = match token.refresh_token() {
        Some(rt) => rt.secret().to_string(),
        None => {
            eprintln!(
                "sign-in: token response carried no refresh_token; is the offline_access scope granted on the Clerk app?"
            );
            return Err(SignInError::ClerkRejected);
        }
    };
    let access_token = token.access_token().secret().to_string();
    let expires_in = token
        .expires_in()
        .map(|d| d.as_secs() as i64)
        .unwrap_or(3600);

    Ok(SessionTokens {
        access_token,
        refresh_token,
        access_expires_at: now_unix() + expires_in,
    })
}

/// Fetch the account (id, email, name) from Clerk's userinfo endpoint.
pub fn fetch_account(access_token: &str) -> Result<AccountInfo, SignInError> {
    let http = http_client(None).map_err(|_| SignInError::CannotReachClerk)?;
    let response = http
        .get(config::userinfo_url())
        .bearer_auth(access_token)
        .send()
        .map_err(|_| SignInError::CannotReachClerk)?;

    let status = response.status();
    if !status.is_success() {
        eprintln!(
            "sign-in: userinfo endpoint returned HTTP {}",
            status.as_u16()
        );
        return Err(SignInError::ClerkRejected);
    }

    // Parse from text so no `json` feature is needed. The raw body is never
    // printed; a parse error prints only its position and the expected type.
    let body = response.text().map_err(|_| SignInError::CannotReachClerk)?;
    let claims: UserinfoClaims = serde_json::from_str(&body).map_err(|e| {
        eprintln!("sign-in: userinfo response did not parse: {e}");
        SignInError::ClerkRejected
    })?;
    if claims.sub.trim().is_empty() {
        eprintln!("sign-in: userinfo response had no subject id");
        return Err(SignInError::ClerkRejected);
    }

    let email = field(&claims.email).to_string();
    let name = claims.resolved_name();
    Ok(AccountInfo {
        id: claims.sub,
        email,
        name,
    })
}

/// Does Clerk answer? Asked once at the very start of every sign-in, before a
/// listener is bound or a browser is opened
/// (docs/decisions/0003-sign-in.md AC-15).
///
/// This is the whole of what "online" means to this feature. It deliberately
/// does not ask Windows whether a network exists: a machine behind a hotel
/// portal or a filtering work network is connected and still cannot sign in.
/// The answer is never cached, because the point is that conditions change.
///
/// Only two outcomes matter, answered or not, so the body is thrown away.
pub fn clerk_answers() -> bool {
    let timeout = Duration::from_secs(config::REACH_CHECK_TIMEOUT_SECS);
    let Ok(http) = http_client(Some(timeout)) else {
        return false;
    };
    match http.get(config::discovery_url()).send() {
        Ok(response) if response.status().is_success() => true,
        Ok(response) => {
            eprintln!(
                "sign-in: Clerk's discovery document returned HTTP {}",
                response.status().as_u16()
            );
            false
        }
        Err(_) => {
            eprintln!("sign-in: Clerk did not answer within the reachability check");
            false
        }
    }
}

/// Trade the stored refresh token for a fresh access token, without the person
/// doing anything. Clerk may rotate the refresh token; if it sends a new one we
/// keep that, otherwise the existing one stays valid.
pub fn refresh_tokens(refresh_token: &str) -> Result<SessionTokens, RefreshError> {
    let token_url = TokenUrl::new(config::token_url()).map_err(|_| RefreshError::Unreachable)?;
    let client = BasicClient::new(ClientId::new(config::OAUTH_CLIENT_ID.to_string()))
        .set_auth_type(AuthType::RequestBody)
        .set_token_uri(token_url);

    let timeout = Duration::from_secs(config::CLERK_CALL_TIMEOUT_SECS);
    let http = http_client(Some(timeout)).map_err(|_| RefreshError::Unreachable)?;

    let token = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
        .request(&http)
        .map_err(classify_refresh_error)?;

    let rotated = token
        .refresh_token()
        .map(|rt| rt.secret().to_string())
        .unwrap_or_else(|| refresh_token.to_string());
    let expires_in = token
        .expires_in()
        .map(|d| d.as_secs() as i64)
        .unwrap_or(3600);

    Ok(SessionTokens {
        access_token: token.access_token().secret().to_string(),
        refresh_token: rotated,
        access_expires_at: now_unix() + expires_in,
    })
}

/// Tell Clerk the refresh token is finished with. Best effort: sign-out has
/// already succeeded locally by the time this runs, and it must never fail or
/// block a person signing out with no network
/// (docs/decisions/0003-sign-in.md AC-15).
pub fn revoke_refresh_token(refresh_token: &str) {
    let timeout = Duration::from_secs(config::CLERK_CALL_TIMEOUT_SECS);
    let Ok(http) = http_client(Some(timeout)) else {
        return;
    };
    // RFC 7009 form post. A public client identifies itself with `client_id`
    // and no secret. The token itself is never printed, here or anywhere.
    let response = http
        .post(config::revoke_url())
        .form(&[
            ("token", refresh_token),
            ("token_type_hint", "refresh_token"),
            ("client_id", config::OAUTH_CLIENT_ID),
        ])
        .send();

    match response {
        Ok(r) if r.status().is_success() => {}
        Ok(r) => eprintln!("sign-out: revocation returned HTTP {}", r.status().as_u16()),
        Err(_) => eprintln!("sign-out: could not reach Clerk to revoke the session"),
    }
}

#[derive(Deserialize)]
struct UserinfoClaims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    given_name: Option<String>,
    #[serde(default)]
    family_name: Option<String>,
}

impl UserinfoClaims {
    /// `name` if Clerk sent one, otherwise given and family joined, otherwise
    /// empty (the interface falls back to the email local part).
    fn resolved_name(&self) -> String {
        let name = field(&self.name);
        if !name.is_empty() {
            return name.to_string();
        }
        format!("{} {}", field(&self.given_name), field(&self.family_name))
            .trim()
            .to_string()
    }
}

/// A claim string with `null` and missing both treated as empty, and trimmed.
fn field(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("").trim()
}

fn http_client(timeout: Option<Duration>) -> Result<oauth2::reqwest::blocking::Client, ()> {
    let mut builder = oauth2::reqwest::blocking::Client::builder()
        .redirect(oauth2::reqwest::redirect::Policy::none());
    if let Some(limit) = timeout {
        builder = builder.timeout(limit);
    }
    builder.build().map_err(|_| ())
}

fn classify_token_error<RE, T>(err: RequestTokenError<RE, T>) -> SignInError
where
    RE: std::error::Error + 'static,
    T: oauth2::ErrorResponse + 'static,
{
    match &err {
        // Could not complete the HTTP call at all.
        RequestTokenError::Request(e) => {
            eprintln!("sign-in: could not reach the token endpoint: {e}");
            SignInError::CannotReachClerk
        }
        // Reached Clerk; it returned a standard OAuth error object. Debug of
        // this type is the error code, description and uri: no token.
        RequestTokenError::ServerResponse(resp) => {
            eprintln!("sign-in: token endpoint rejected the exchange: {resp:?}");
            SignInError::ClerkRejected
        }
        // A 2xx body that did not parse. The raw bytes are deliberately not
        // printed, since a success body would contain tokens.
        RequestTokenError::Parse(e, _raw) => {
            eprintln!("sign-in: token response did not parse: {e}");
            SignInError::ClerkRejected
        }
        RequestTokenError::Other(msg) => {
            eprintln!("sign-in: token exchange failed: {msg}");
            SignInError::ClerkRejected
        }
    }
}

/// The one distinction AC-14 and AC-16 hang on: did Clerk answer and say no,
/// or did we never reach it? Anything Clerk answered with is a refusal, and
/// signs the person out. Everything else leaves the session alone.
fn classify_refresh_error<RE, T>(err: RequestTokenError<RE, T>) -> RefreshError
where
    RE: std::error::Error + 'static,
    T: oauth2::ErrorResponse + 'static,
{
    match &err {
        RequestTokenError::Request(e) => {
            eprintln!("refresh: could not reach the token endpoint: {e}");
            RefreshError::Unreachable
        }
        // Debug of this type is the OAuth error code, description and uri. No
        // token is in it.
        RequestTokenError::ServerResponse(resp) => {
            eprintln!("refresh: Clerk refused the refresh token: {resp:?}");
            RefreshError::Refused
        }
        // A 2xx body we could not read. Clerk answered, but not with a refusal,
        // so the session is left in place rather than ending it on a guess. The
        // raw bytes are deliberately not printed: a success body holds tokens.
        RequestTokenError::Parse(e, _raw) => {
            eprintln!("refresh: token response did not parse: {e}");
            RefreshError::Unreachable
        }
        RequestTokenError::Other(msg) => {
            eprintln!("refresh: refresh failed: {msg}");
            RefreshError::Unreachable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims(name: &str, given: &str, family: &str) -> UserinfoClaims {
        UserinfoClaims {
            sub: "u".into(),
            email: None,
            name: opt(name),
            given_name: opt(given),
            family_name: opt(family),
        }
    }

    fn opt(s: &str) -> Option<String> {
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }

    #[test]
    fn the_reachability_check_asks_the_same_clerk_domain_as_everything_else() {
        // It must be the dependency being tested, not the internet in general,
        // and it must never carry a token or a query.
        let url = config::discovery_url();
        assert!(url.starts_with(&format!("https://{}/", config::CLERK_DOMAIN)));
        assert!(url.ends_with("/.well-known/openid-configuration"));
        assert!(!url.contains('?'));
    }

    #[test]
    fn error_codes_are_distinct_and_stable() {
        let all = [
            SignInError::BrowserClosed,
            SignInError::TimedOut,
            SignInError::SecurityCheckFailed,
            SignInError::ClerkRejected,
            SignInError::CannotReachClerk,
        ];
        let codes: Vec<&str> = all.iter().map(|e| e.code()).collect();
        let mut unique = codes.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(codes.len(), unique.len());
        assert_eq!(SignInError::TimedOut.code(), "timed_out");
    }

    #[test]
    fn resolved_name_prefers_name_then_given_family() {
        assert_eq!(
            claims("Sam Rivera", "Ignore", "Me").resolved_name(),
            "Sam Rivera"
        );
        assert_eq!(claims("  ", "Sam", "Rivera").resolved_name(), "Sam Rivera");
        assert_eq!(claims("", "", "").resolved_name(), "");
    }

    #[test]
    fn null_claims_deserialise_to_empty() {
        let json = r#"{"sub":"user_1","email":null,"name":null}"#;
        let parsed: UserinfoClaims = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.sub, "user_1");
        assert_eq!(field(&parsed.email), "");
        assert_eq!(parsed.resolved_name(), "");
    }
}
