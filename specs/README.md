# Feature notes

These notes document implemented behavior and known limitations. Check the
implementation when a note and the code disagree. Keep proposals clearly labeled
as proposals rather than describing them as current behavior.

| Note | Status | Primary code | Decision |
|---|---|---|---|
| [GitHub Actions](./1-github-actions.md) | Implemented | `.github/workflows/` | — |
| [In-memory caching](./2-caching.md) | Implemented; missing TTL can panic at startup | `src/state.rs`, `src/cache/inmemory/` | [ADR-0007](../docs/adr/0007-process-local-moka-cache.md) |
| [XSS sanitization](./3-xss-sanitization.md) | Implemented | `src/utils.rs` | [ADR-0005](../docs/adr/0005-sanitize-authored-blog-html.md) |
| [CSRF protection](./4-csrf-protection.md) | Implemented; unauthorized HTML response has no explicit status code | `src/handler/auth/csrf.rs`, `src/handler/admin/` | [ADR-0004](../docs/adr/0004-double-submit-csrf-protection.md) |
| [Login rate limiting](./5-rate-limiting.md) | Implemented; reverse-proxy IP limitation | `src/routes.rs` | — |
| [JWT authentication](./6-jwt-authentication.md) | Implemented; unauthorized page has no explicit status code | `src/handler/auth/`, `src/model/auth.rs` | [ADR-0003](../docs/adr/0003-password-and-jwt-authentication.md) |

`0-template.md` is a template for future feature notes. `references.md` is a
collection of implementation references.
