# Accept dependency advisories by root cause with documented revisit triggers

`cargo deny` reports vulnerabilities and unmaintained crates from the RustSec
advisory database. All currently ignored advisories trace to three root
dependencies, two of which are already at their newest release, so the risk is
accepted with explicit revisit triggers instead of attempting immediate upgrades.

## Root causes and risk assessment

| Root dependency | Version state | Advisories pulled in |
|---|---|---|
| `libsql` | 0.10.0 (latest; no upgrade available) | rustls-pemfile and bincode unmaintained; rustls-webpki 0.102.8, rustls 0.22.4, and h2 0.3.27 vulnerabilities |
| `askama` | 0.16.1 (latest) | basic-toml unmaintained; used only by the proc-macro at compile time, no runtime exposure |
| `jsonwebtoken` | 11.1.0 (latest) | rsa (Marvin Attack, unpatched) and crypto-bigint timing variability |

The libsql chain advisories affect only the stale copies: the newer rustls
0.23.46 and h2 0.4.20 also present in the tree are already patched. The
jsonwebtoken advisories are unreachable at runtime because the app signs and
verifies HS256 (HMAC) tokens only; rsa and the ECDSA stack are never used.

## Decision

1. **Accept and document** the libsql and askama advisories in the `deny.toml`
   ignore list, each with the root cause in its `reason`. A single future
   libsql release that updates its TLS stack lifts ten advisories at once, so
   no interim action is worth the churn.
2. **Replace jsonwebtoken** with a minimal HS256 implementation built on
   `hmac`, `sha2`, and `base64` (see `src/handler/auth/mod.rs`). This removes
   `rsa`, `crypto-bigint`, and the ECDSA tree from the binary entirely,
   lifting RUSTSEC-2023-0071 and RUSTSEC-2026-0387. The wire format is
   unchanged (standard unpadded base64url HS256), and the explicit `alg`
   header check prevents algorithm confusion. This supersedes the crate
   choice in [ADR-0003](./0003-password-and-jwt-authentication.md); ADR-0003's
   authentication design itself is unchanged.

## Revisit triggers

- A libsql release that updates rustls/hyper — re-run `cargo deny`, lift the
  libsql-blocked entries.
- An askama release replacing basic-toml — lift RUSTSEC-2026-0374.
- A requirement to sign tokens with RSA/EC keys — cannot return to
  jsonwebtoken as-is; re-evaluate its successor or a maintained alternative.

## Consequences

- The ignore list stays short and reviewable: every entry names its root cause.
- A newer release of a root dependency can lift several advisories with one
  upgrade; the `advisory-not-detected` warning makes stale entries visible.
