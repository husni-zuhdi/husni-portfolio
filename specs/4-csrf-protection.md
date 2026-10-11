# CSRF protection

**Status: Implemented.** Current behavior is described below; the original
proposal's expected `403` response is not how the current handlers respond.

Decision: [ADR-0004](../docs/adr/0004-double-submit-csrf-protection.md).

## Behavior

- Double-submit cookie validation was chosen to protect cookie-authenticated
  HTMX writes without adding a heavy session-management crate. `SameSite=Strict`
  is defense-in-depth, not a substitute for the CSRF token check.
- Successful login creates a random 32-byte CSRF token represented as 64
  hexadecimal characters and sets it in `_csrf_token`.
- The browser-readable CSRF cookie is sent with `Secure; SameSite=Strict; Path=/`.
  The JWT cookie is `HttpOnly; Secure; SameSite=Strict`.
- The Admin area base template listens for HTMX's `htmx:configRequest` event and adds
  the cookie value as the `X-CSRF-Token` request header.
- Blog, Tag, and Talk write requests made through the Admin area, along with
  `DELETE /logout`, require a valid JWT and an exact match between the CSRF cookie
  and request header. Administrator sign-in itself is not CSRF-checked.
- Logout clears both authentication cookies.
- A missing or mismatched token returns the unauthorized HTML template via
  `Html<String>`. The handler does not set an explicit HTTP status code, so this
  response currently uses Axum's default success status despite the template's
  “401 Unauthorized” content.

Cookie extraction splits the `Cookie` header into segments and matches cookie
names, so the `_csrf_token` value cannot be mistaken for the `token` JWT.

## Main paths

- Token generation, verification, and cookie formatting:
  `src/handler/auth/csrf.rs`
- Shared cookie extraction and JWT verification: `src/handler/auth/mod.rs`
- Login and logout: `src/handler/auth/operations.rs`
- Admin area write operations: `src/handler/admin/`
- HTMX request header setup: `templates/admin/admin_base.html`

## Tests

CSRF token and cookie checks are tested in `src/handler/auth/csrf.rs`; shared
cookie parsing and JWT behavior are tested in `src/handler/auth/mod.rs`. These are
module-level tests run by `task test`.

## References

- [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
- [HTMX `htmx:configRequest` event](https://htmx.org/events/#htmx:configRequest)
