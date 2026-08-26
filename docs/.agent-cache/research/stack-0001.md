# EchoScribe v2 Stack Research - August 2026

## 1. Desktop App Frameworks (Windows → macOS)

**Tauri 2.0**
- Language: Rust (backend) + Web frontend (TypeScript/React)
- Maintenance: Active (v2.0 stable late 2025)
- Global hotkeys: Native via tauri-plugin-global-hotkey ([github.com/tauri-apps/global-hotkey](https://github.com/tauri-apps/global-hotkey))
- Text injection: Not natively built-in; requires Windows API wrapper or native plugin
- App size: ~50-100 MB (WebView2 + minimal Rust)
- Notes: WebView2 on Windows (larger than native), WebKit on macOS; rendering variations across platforms
- Source: [PkgPulse 2026 guide](https://www.pkgpulse.com/guides/best-desktop-app-frameworks-2026)

**Electron**
- Language: JavaScript/TypeScript + Chromium
- Maintenance: Mature, stable
- Global hotkeys: Native via electron-globalshortcut
- Text injection: Not built-in; requires native module
- App size: ~150-300 MB (bundled Chromium + Node.js)
- Notes: Largest footprint; most mature ecosystem; predictable cross-platform
- Source: [PkgPulse 2026 guide](https://www.pkgpulse.com/guides/best-desktop-app-frameworks-2026)

**WinUI 3 / .NET MAUI**
- Language: C# + XAML
- Maintenance: Active (Microsoft, part of Windows App SDK)
- Global hotkeys: Supported via P/Invoke or Windows Runtime APIs
- Text injection: Native Windows API access (SendInput, SetClipboard)
- App size: ~20-50 MB (minimal, leverages OS libraries)
- Notes: Windows-only (WinUI 3) or cross-platform (MAUI); true native performance
- Source: [GitHub WinUI hotkeys gist](https://gist.github.com/jkdba/db450898f0ad9b6ebf4280c91c003fb7), [bool.dev interview questions](https://bool.dev/blog/detail/part11-desktop-dotnet-interview-questions)

**Flutter Desktop**
- Language: Dart
- Maintenance: Active, cross-platform
- Global hotkeys: Third-party plugins exist (community-maintained)
- Text injection: Could not verify native support
- App size: ~40-80 MB
- Source: [PkgPulse 2026 guide](https://www.pkgpulse.com/guides/best-desktop-app-frameworks-2026)

---

## 2. Cloud Speech-to-Text APIs

| Provider | Pricing (per min) | Free Tier | Languages | Latency |
|----------|------------------|-----------|-----------|---------|
| OpenAI Whisper | $0.006 (batch) / free (OSS) | Unlimited (open-source) | 57+ | ~1-2s (file) |
| Deepgram | $0.0043 (batch) / $0.0077 (stream) | $200 credits | 36+ | <200ms (streaming) |
| AssemblyAI | ~$0.006 ($0.37/hr) | None explicit | 99+ | ~5-10s (typical) |
| Google Cloud | $0.016 | None stated | 125+ | ~1-2s (batch) |
| Azure AI | $0.017 / $1/hr | 5 hr/month | 100+ | ~1-2s (batch) |

- OpenAI is free via open-source model; API adds cost
- Deepgram has most generous free credits; highest language coverage is Google Cloud
- Sources: [FutureAGI guide](https://futureagi.com/blog/speech-to-text-apis-in-2026-benchmarks-pricing-developer-s-decision-guide/), [AssemblyAI alternatives](https://www.assemblyai.com/blog/google-cloud-speech-to-text-alternatives)

---

## 3. Native Desktop Auth Providers

| Provider | Free Tier | Cost Beyond Free | Desktop Support | Native SDK |
|----------|-----------|------------------|-----------------|-----------|
| Clerk | 50,000 MRU | $0.02/MRU | React/Next.js focused; could not verify native desktop | Limited |
| Auth0 | 25,000 MAU | $35+/month | Enterprise; OAuth/OIDC standard | Backend SDKs only |
| Supabase Auth | 50,000 MAU | $0.00325/MAU | Open-source GoTrue; PKCE-ready | JS/Dart/Python |
| Firebase Auth | 50,000 MAU | Pay-per-use | Mobile (iOS/Android/C++/Flutter) strong | C++, Dart, multi-platform |
| WorkOS | 1M MAU (user mgmt) | +$125/SSO connection | Enterprise; no explicit desktop mention | Backend SDKs (.NET, Node, Go) |

- **All use OAuth2+PKCE for secure native flows**, but documentation is web/mobile-focused
- Supabase Auth is open-source (can self-host) and cheapest at scale
- Firebase has strongest C++ SDK for desktop consideration
- Sources: [DesignRevision comparison](https://designrevision.com/blog/auth-providers-compared), [BuildMVPFast pricing](https://www.buildmvpfast.com/api-costs/authentication), [Merginit free auth comparison](https://merginit.com/blog/13062026-free-auth-identity-providers-comparison)

---

*Research completed August 25, 2026. Information current as of search dates.*
