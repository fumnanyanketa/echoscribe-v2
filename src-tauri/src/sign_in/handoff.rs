//! The browser handoff. Rust drives all of it: it makes a PKCE challenge and a
//! random state, binds a one-shot listener on `127.0.0.1` on a free port,
//! opens the system browser to Clerk, and waits for Clerk to redirect back
//! with the authorization code. The interface only asks it to start
//! (docs/decisions/0003-sign-in.md).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use oauth2::basic::BasicClient;
use oauth2::url::Url;
use oauth2::{AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl};
use tiny_http::{Header, Request, Response, Server};

use super::config;

/// A sign-in in flight: the live listener, the redirect URI that was sent to
/// Clerk, and the two per-attempt secrets held in memory only.
pub struct Handoff {
    server: Server,
    pub redirect_uri: String,
    pub csrf_state: String,
    pub pkce_verifier: String,
}

/// Why `start` could not even get the browser open. Distinct from a sign-in
/// that starts and then fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartError {
    NoLoopbackPort,
    BrowserFailed,
    Internal,
}

impl StartError {
    pub fn code(self) -> &'static str {
        match self {
            Self::NoLoopbackPort => "no_loopback_port",
            Self::BrowserFailed => "browser_failed",
            Self::Internal => "internal",
        }
    }
}

/// What came back on the loopback listener.
#[derive(Debug, PartialEq, Eq)]
pub enum Callback {
    /// The authorization code, with a state that matched this attempt.
    Code(String),
    /// A callback arrived but its state did not match: treat as an attack.
    StateMismatch,
    /// Clerk redirected back with an error (the standard OAuth `error` code, if
    /// present) or the callback was malformed.
    ClerkError(Option<String>),
    /// The person cancelled, or closed the browser and gave up.
    Abandoned,
    /// No callback within the timeout.
    TimedOut,
}

/// Bind the listener, build the authorize URL, open the browser. Returns once
/// the browser has been launched.
pub fn start() -> Result<Handoff, StartError> {
    let server = Server::http("127.0.0.1:0").map_err(|_| StartError::NoLoopbackPort)?;
    let port = server
        .server_addr()
        .to_ip()
        .ok_or(StartError::NoLoopbackPort)?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}{}", config::REDIRECT_PATH);

    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

    let client = BasicClient::new(ClientId::new(config::OAUTH_CLIENT_ID.to_string()))
        .set_auth_uri(AuthUrl::new(config::authorize_url()).map_err(|_| StartError::Internal)?)
        .set_token_uri(TokenUrl::new(config::token_url()).map_err(|_| StartError::Internal)?)
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.clone()).map_err(|_| StartError::Internal)?,
        );

    let mut request = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(challenge);
    for scope in config::SCOPES {
        request = request.add_scope(Scope::new((*scope).to_string()));
    }
    let (auth_url, csrf) = request.url();

    open::that(auth_url.as_str()).map_err(|_| StartError::BrowserFailed)?;

    Ok(Handoff {
        server,
        redirect_uri,
        csrf_state: csrf.secret().to_string(),
        pkce_verifier: verifier.secret().to_string(),
    })
}

/// Block until the callback arrives, the person cancels (`cancel` flips true),
/// or the timeout runs out. Serves a plain page to the browser either way.
pub fn wait_for_callback(handoff: &Handoff, cancel: &AtomicBool) -> Callback {
    let deadline = Instant::now() + Duration::from_secs(config::SIGN_IN_TIMEOUT_SECS);
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Callback::Abandoned;
        }
        let now = Instant::now();
        if now >= deadline {
            return Callback::TimedOut;
        }
        let slice = (deadline - now).min(Duration::from_millis(400));
        match handoff.server.recv_timeout(slice) {
            Ok(Some(request)) => match classify(request.url(), &handoff.csrf_state) {
                Parsed::Match(code) => {
                    serve(request, SUCCESS_PAGE);
                    return Callback::Code(code);
                }
                Parsed::StateMismatch => {
                    serve(request, FAILURE_PAGE);
                    return Callback::StateMismatch;
                }
                Parsed::ClerkError(oauth_error) => {
                    serve(request, FAILURE_PAGE);
                    return Callback::ClerkError(oauth_error);
                }
                Parsed::NotTheCallback => serve_status(request, 404),
            },
            Ok(None) => {}
            Err(_) => return Callback::TimedOut,
        }
    }
}

#[derive(Debug)]
enum Parsed {
    Match(String),
    StateMismatch,
    /// Carries the standard OAuth `error` code when Clerk sent one.
    ClerkError(Option<String>),
    NotTheCallback,
}

fn classify(raw_url: &str, expected_state: &str) -> Parsed {
    let Ok(url) = Url::parse(&format!("http://127.0.0.1{raw_url}")) else {
        return Parsed::NotTheCallback;
    };
    if url.path() != config::REDIRECT_PATH {
        return Parsed::NotTheCallback;
    }

    let mut code = None;
    let mut state = None;
    let mut oauth_error = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => oauth_error = Some(value.into_owned()),
            _ => {}
        }
    }

    if oauth_error.is_some() {
        return Parsed::ClerkError(oauth_error);
    }
    match (code, state) {
        (Some(code), Some(state)) if !code.is_empty() => {
            if state == expected_state {
                Parsed::Match(code)
            } else {
                Parsed::StateMismatch
            }
        }
        _ => Parsed::ClerkError(None),
    }
}

fn serve(request: Request, body: &str) {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
        .expect("static header is valid");
    let _ = request.respond(Response::from_string(body).with_header(header));
}

fn serve_status(request: Request, status: u16) {
    let _ = request.respond(Response::from_string("").with_status_code(status));
}

const SUCCESS_PAGE: &str = "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
<title>EchoScribe</title>\
<body style=\"font-family:system-ui,sans-serif;text-align:center;padding:3rem;color:#14161a\">\
<h1>Signed in</h1><p>You can close this tab and return to EchoScribe.</p></body></html>";

const FAILURE_PAGE: &str = "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
<title>EchoScribe</title>\
<body style=\"font-family:system-ui,sans-serif;text-align:center;padding:3rem;color:#14161a\">\
<h1>Sign-in did not finish</h1><p>Return to EchoScribe and try again.</p></body></html>";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_state_yields_the_code() {
        match classify("/callback?code=abc123&state=xyz", "xyz") {
            Parsed::Match(code) => assert_eq!(code, "abc123"),
            _ => panic!("expected a match"),
        }
    }

    #[test]
    fn wrong_state_is_a_mismatch() {
        assert!(matches!(
            classify("/callback?code=abc&state=WRONG", "xyz"),
            Parsed::StateMismatch
        ));
    }

    #[test]
    fn clerk_error_in_query_is_carried() {
        match classify("/callback?error=access_denied&state=xyz", "xyz") {
            Parsed::ClerkError(Some(code)) => assert_eq!(code, "access_denied"),
            other => panic!("expected a carried error, got {other:?}"),
        }
    }

    #[test]
    fn other_paths_are_ignored() {
        assert!(matches!(
            classify("/favicon.ico", "xyz"),
            Parsed::NotTheCallback
        ));
        assert!(matches!(classify("/", "xyz"), Parsed::NotTheCallback));
    }

    #[test]
    fn callback_without_a_code_is_treated_as_rejected() {
        assert!(matches!(
            classify("/callback?state=xyz", "xyz"),
            Parsed::ClerkError(None)
        ));
    }

    #[test]
    fn start_error_codes_are_distinct() {
        assert_ne!(
            StartError::NoLoopbackPort.code(),
            StartError::BrowserFailed.code()
        );
    }
}
