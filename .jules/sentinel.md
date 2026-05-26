## 2025-01-24 - [Secure Cookie Enhancement]
**Vulnerability:** Cookies (session and device) were missing the `Secure` flag even when the application was configured for HTTPS.
**Learning:** In a dual-mode application (local/remote), cookie attributes should dynamically adapt to the environment. Hardcoding `Secure=true` would break local-network usage over HTTP, while omitting it weakens security for remote HTTPS usage.
**Prevention:** Use configuration signals (like `public_url` prefix) to determine when to upgrade cookie security flags.
