# Clerk OAuth/OIDC Browser Session Sign-Out

**Date:** 2026-10-05

Research findings on Clerk's OAuth 2.0 / OIDC provider (`/oauth/authorize` endpoint) capabilities for browser session management and sign-out.

## Question 1: OpenID `prompt` Parameter Support

**Answer:** Not explicitly documented in Clerk's official documentation.

The OpenID Connect `prompt` parameter (`prompt=login`, `prompt=select_account`, `prompt=consent`) is a standard OIDC feature, but I could not find documentation confirming whether Clerk's `/oauth/authorize` endpoint accepts this parameter. The Clerk Backend API reference at https://clerk.com/docs/reference/backend-api shows OAuth Applications and OAuth Access Tokens endpoints, but does not detail OIDC provider parameters for the authorize endpoint.

**Citation:** https://clerk.com/docs/reference/backend-api/tag/OAuth-Applications — (no specific parameter documentation found)

## Question 2: End Session Endpoint (RP-Initiated Logout)

**Answer:** No `end_session_endpoint` in the OpenID discovery document; no documented `/sign-out` URL; no documented Frontend API call to end Account Portal sessions.

You noted that the discovery document at `/.well-known/openid-configuration` currently shows **NO** `end_session_endpoint`. Clerk's documentation does not expose:
- An RP-initiated logout endpoint (missing `end_session_endpoint`)
- A documented Account Portal `/sign-out` URL to end the browser session
- A Frontend API method to revoke the Account Portal's own sessions from outside

This means signing out from the OAuth client side (revoking the refresh token) does not clear Clerk's Account Portal browser session cookies, leaving the user auto-authenticated on the next authorize request.

**Citation:** https://clerk.com/docs/reference/backend-api — (no end_session_endpoint documented; no documented Portal sign-out URL found)

## Question 3: Backend API Session Revocation

**Answer:** Sessions endpoint exists in Clerk's Backend API; specific revocation endpoints not found in read documentation.

Clerk's Backend API includes a Sessions tag at https://clerk.com/docs/reference/backend-api/tag/Sessions, indicating session management capabilities exist. The Backend API is protected and requires authentication (API key). **Without a private API key and secret, a public OAuth client cannot call the Backend API.** The app holds only a public `client_id`, so this endpoint is not usable for EchoScribe without a confidential backend server.

**Citation:** https://clerk.com/docs/reference/backend-api/tag/Sessions — (Sessions endpoint exists; requires Backend API authentication)

## Question 4: Account Portal Auto-Continue Setting

**Answer:** Unknown / not documented.

Clerk's documentation does not expose a setting to force the Account Portal to show the account chooser or email field on every authorize request, overriding the auto-continue-with-existing-session behavior. This may exist in the Clerk Dashboard instance settings, but it is not documented in the public API references checked.

**Citation:** Not found in Clerk's official docs at https://clerk.com/docs

## Summary

- **Prompt parameter:** Not explicitly documented
- **End session endpoint:** No end_session_endpoint in discovery; no documented Portal sign-out; no documented Frontend API for session revocation
- **Backend API session revocation:** Exists but requires confidential authentication (not usable by public OAuth clients)
- **Auto-continue override setting:** Unknown / not documented

The browser session persistence issue cannot be resolved through documented Clerk public APIs from a public OAuth client. Workarounds would require:
1. Contacting Clerk support for an undocumented feature or setting
2. Building a confidential backend proxy that calls Clerk's Backend API
3. Accepting the auto-continue behavior as expected for OAuth clients without an end_session_endpoint
