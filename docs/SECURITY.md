# Security

## Trust Model

TSSP v1 is designed for trusted local networks. The daemon does not implement
authentication yet. Do not expose `tsspd` to the public internet.

## Implemented Protections

- QR session token validation requires unpadded base64url text sized for 128 bits
  of entropy.
- Session state is single-use at the domain layer.
- Filenames are never used directly as storage paths.
- The web fallback sets `X-Content-Type-Options: nosniff`.
- The web fallback sets a restrictive Content Security Policy.

## Reporting Vulnerabilities

Open a private security advisory on the project repository or contact the
maintainer through the repository owner profile.
