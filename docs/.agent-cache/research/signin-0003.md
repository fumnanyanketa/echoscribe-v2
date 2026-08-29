# EchoScribe sign-in (record 0003) research - 2026-08-28

cached 2026-08-28, reuse for 30 days

## Research scope

Desktop Tauri app (Windows, Rust core + plain-JS WebView). No backend server. Already chosen: Clerk for sign-in. Must stay signed in across restarts (silent refresh), work offline once signed in, support passwordless email OTC.

## Findings

### 1. Native desktop app support

**Official status:** Clerk supports native desktop apps but does NOT have first-party official SDKs for Tauri or Electron documented as of 2026-08.
- Clerk has community-maintained Tauri plugin (mentioned in search, exact URL not indexed)
- Clerk has native mobile SDKs for iOS and Expo (React Native)
- No dedicated Electron guide found in official docs

**Source:** Clerk Docs search (2026-08-28)

### 2. OAuth 2.0 Identity Provider / OAuth applications

**Does Clerk support OAuth applications:** YES. Clerk can act as an "OAuth 2.0 and OpenID Connect (OIDC) Identity Provider" for third-party apps.
- URL: https://clerk.com/docs/advanced-usage/clerk-idp

**PKCE support:** YES. PKCE (Proof Key for Code Exchange) is now supported for custom OAuth providers (added 2025-11-12). Public client mode is available.
- URL: https://clerk.com/changelog/2025-11-12-pkce-support-custom-oauth

**Refresh tokens:** YES, but only explicitly confirmed for OIDC flows (which issue access token, ID token, AND refresh token).
- Access/refresh token lifetimes: **NOT FOUND** in official docs
- Scopes available: `openid`, `profile`, `email` documented; specific user fields (email, name, username, picture, org info) returned based on scopes

**Source:** Clerk IdP docs (2026-08-28); PKCE changelog (2025-11-12)

### 3. Browser-to-app handoff

**Loopback redirect (http://127.0.0.1:PORT):** YES, confirmed supported.
- Clerk CLI blog explicitly recommends "starting a temporary HTTP listener on `127.0.0.1:0`" with OAuth Code + PKCE flow
- Localhost works "in development" when sharing same origin

**Custom URL schemes / deep links:** YES, supported.
- Documentation states: "The full url value can be prefixed with https:// or a custom scheme, for example, https://my-app.com/oauth-callback or my-app://oauth-callback"
- Custom schemes like `myapp://` are explicitly listed as allowed redirect URLs

**Redirect URL constraints:** Unknown for non-web apps specifically; no explicit desktop/native restrictions found in docs.

**Source:** Clerk CLI blog: "Adding Clerk auth to your CLI" (exact date not provided); redirect URLs page (https://clerk.com/docs/guides/development/customize-redirect-urls)

### 4. Frontend API + publishable key (long-lived client without clerk-js)

**Status:** NOT FOUND in official documentation. 

Session token retrieval (`getToken()`) and session refresh are documented, but these are part of the JavaScript SDK, not a raw HTTP API for non-browser clients. No explicit support found for a non-browser client (Rust core) establishing a session using only a publishable key and calling raw endpoints.

**Token refresh mechanism:** Session tokens auto-refresh every 50 seconds when clerk-js is active. No details on how to implement this without the SDK.

**Recommendation:** May require custom implementation using OAuth token endpoints directly (Authorization Code + PKCE flow), or using Clerk's Backend API with M2M auth (requires private API key, which violates "secrets never cross into interface" rule if stored in WebView context).

### 5. Community Tauri + Clerk integrations

**Status:** Community Tauri plugin mentioned but not indexed. No public Tauri or Electron + Clerk example projects found in search results.

**Recommendation:** Check Clerk's community links or GitHub for unofficial examples; may need to build custom integration.

### 6. Account Portal configuration

**Email one-time-code (passwordless):** UNCLEAR. Documentation mentions sign-in is "controlled by instance settings" but does not specify which methods are supported in the Dashboard.

**Name field at sign-up:** UNCLEAR. Documentation mentions sign-up page renders UI but does not detail customizable fields.

**Redirect behavior:** Confirmed that Account Portal redirects after sign-in/sign-up, but redirect_url configuration details not found. "Cannot be customized beyond options in Clerk Dashboard."

**Source:** Account Portal overview (2026-08-28)

### 7. Device Authorization Grant (RFC 8628)

**Status:** MENTIONED BUT NOT PRIMARY.

Clerk's CLI blog explicitly states: "Device flow (RFC 8628) is an alternative for CI, headless servers, and environments where the CLI can't open a browser... [but this is] outside the scope" of their primary recommendation.

**Official support unclear:** Device code flow is acknowledged as existing but not documented as a first-class Clerk feature. Authorization Code + PKCE + localhost is the recommended pattern.

**Source:** Clerk blog: "Adding Clerk auth to your CLI" (exact date not provided)

## Open unknowns (stop and confirm with Clerk)

1. **Refresh token lifetime:** Session token is 60 seconds, but refresh token lifetime unknown
2. **Authorization Code vs OIDC:** Does the Authorization Code flow (standard OAuth) issue refresh tokens, or only OIDC?
3. **Rust core token storage:** How should the Rust core maintain a session after browser-to-app handoff without calling the JavaScript SDK?
4. **Device code flow status:** Is device code flow officially supported? Is it documented?
5. **Account Portal passwordless:** Can Account Portal be configured for email OTC sign-in in the Dashboard?
6. **Official Tauri/Electron docs:** Are there official Clerk guides or examples for desktop frameworks?

## Recommended starting pattern (from available evidence)

- **OAuth 2.0 Authorization Code + PKCE + localhost redirect** (http://127.0.0.1:PORT/callback)
  - Browser handoff: user signs in via Clerk Account Portal, gets authorization code
  - Redirect back to app via loopback URL
  - Rust core exchanges code for access token + refresh token
  - Rust core stores tokens securely (OS credential store or encrypted local file)
  - Automatic refresh before expiry to maintain "signed in across restarts"
- **Alternative for offline/headless:** Device code flow (if officially supported; needs verification)
- **Session management:** Rust core responsible for token refresh; WebView never sees credentials

## References

- Clerk Docs: https://clerk.com/docs
- Clerk IdP (OAuth/OIDC): https://clerk.com/docs/advanced-usage/clerk-idp
- PKCE support announcement: https://clerk.com/changelog/2025-11-12-pkce-support-custom-oauth
- CLI auth pattern: https://clerk.com/blog/adding-clerk-auth-to-your-cli
- Session tokens: https://clerk.com/docs/guides/sessions/session-tokens
- Redirect URLs: https://clerk.com/docs/guides/development/customize-redirect-urls
- Account Portal: https://clerk.com/docs/guides/account-portal/overview
