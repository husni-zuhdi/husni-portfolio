# CSRF Protection

## Goals
Prevent Cross-Site Request Forgery (CSRF) attacks on all state-changing HTMX forms
(login, logout, admin CRUD). An attacker could craft a malicious page that auto-submits
forms to `husni-zuhdi.com` while the admin is logged in. The browser automatically
attaches the JWT cookie, and without CSRF validation, the server cannot distinguish
legitimate from forged requests.

## Criterias
- All state-changing HTMX forms (`hx-post`, `hx-put`, `hx-delete`) include a CSRF token
- The CSRF token is validated server-side on every state-changing request
- The token is tied to the authenticated session (generated at login, cleared at logout)
- The approach uses `hx-headers` for HTMX integration (no additional JS required)
- No heavy session management crate is added (no `axum-sessions`, `tower-sessions`)
- The JWT cookie is upgraded from `SameSite=Lax` to `SameSite=Strict` as a defense-in-depth layer

## Usage
This implementation uses the **Double-Submit Cookie** pattern as recommended by OWASP for
SPA/HTMX backends:

1. On successful login, generate a random CSRF token and set it in a **non-HttpOnly** cookie
   (so HTMX's JavaScript can read it). The JWT cookie remains HttpOnly (inaccessible to JS).
2. HTMX forms include the token via `hx-headers='{"X-CSRF-Token": "<token>"}'`
3. On every state-changing request, the server compares the cookie value with the header value.
   If they don't match, the request is rejected with 403.

This works because:
- An attacker on `evil.com` can cause the browser to **send** the cookie (same-site), but
  **cannot read** the cookie value (Same-Origin Policy), so they cannot set the `X-CSRF-Token`
  header to match.
- The `SameSite=Strict` JWT cookie prevents cross-site cookie submission entirely, adding
  a second layer of defense.

### Token properties
- 32-byte cryptographically random hex string (64 hex characters)
- Generated once per login session (stored as a field in the JWT Claims or a separate cookie)
- Cleared on logout (cookie expired)

### Draft of envars
No new environment variables are required. The CSRF token cookie uses a fixed name
(`_csrf_token`) and the same `Secure`/`SameSite`/`Path` properties as the JWT cookie.

## Flow

### Login — token generation

```mermaid
sequenceDiagram
    participant U as User (Browser)
    participant H as Handler
    participant JWT as JWT Claims

    U->>H: POST /login (email + password)
    H->>H: Validate credentials
    H->>JWT: Create JWT (exp: 3h)
    H->>H: Generate 32-byte random CSRF token
    H-->>U: Set-Cookie: token=<JWT>; HttpOnly; Secure; SameSite=Strict
    H-->>U: Set-Cookie: _csrf_token=<CSRF>; Secure; SameSite=Strict
    H-->>U: HX-Redirect: /admin
```

### Form submission — token validation

```mermaid
sequenceDiagram
    participant U as User (Browser)
    participant H as Handler

    U->>H: POST /admin/blogs/add
    Note over U: Cookie: token=<JWT>, _csrf_token=<CSRF>
    Note over U: Header: X-CSRF-Token: <CSRF>
    H->>H: Read _csrf_token from Cookie header
    H->>H: Read X-CSRF-Token from request header
    alt Token matches
        H->>H: Process request (add blog)
        H-->>U: 200 OK
    else Token missing or mismatch
        H-->>U: 403 Forbidden
    end
```

### CSRF attack — blocked

```mermaid
sequenceDiagram
    participant V as Victim Browser
    participant E as Evil Website
    participant S as husni-zuhdi.com

    E->>V: Serve malicious page with hidden form
    V->>S: POST /admin/blogs/add (auto-sends JWT cookie)
    Note over V: Cookie: token=<JWT> (auto-attached)
    Note over V: Header: X-CSRF-Token: MISSING (JS can't read HttpOnly cookie)
    S->>S: Compare cookie _csrf_token with header X-CSRF-Token
    S-->>V: 403 Forbidden
```

### Logout — token cleared

```mermaid
sequenceDiagram
    participant U as User (Browser)
    participant H as Handler

    U->>H: DELETE /logout
    H-->>U: Set-Cookie: token=; Max-Age=0
    H-->>U: Set-Cookie: _csrf_token=; Max-Age=0
    H-->>U: HX-Redirect: /
```

## Implementation locations

| File | Change |
|---|---|
| `src/handler/auth/mod.rs` | Add `generate_csrf_token()` function (32 random bytes via `rand` or `ring`) |
| `src/handler/auth/operations.rs` | `post_login`: set `_csrf_token` cookie on success. `delete_logout`: clear `_csrf_token` cookie |
| `src/handler/admin/mod.rs` | Add `verify_csrf_token(cookie_header, header_map) -> bool` helper function |
| `src/handler/admin/*/operations.rs` | Call `verify_csrf_token()` at the top of every POST/PUT/DELETE handler |
| `src/model/auth.rs` | Add `csrf_token: String` field to `Claims` struct (optional — token can also live purely in cookie) |
| `templates/base.html` or `admin/admin_base.html` | Add `hx-headers` with CSRF token from cookie via small inline script |
| `templates/admin/**/*.html` | All forms get `hx-headers` attribute pointing to CSRF token |
| `Cargo.toml` | Add `rand` dependency (for secure random token generation) |
| `src/routes.rs` | Ensure `SameSite=Strict` in cookie strings for both JWT and CSRF cookies |

### HTMX hx-headers integration
In `templates/admin/admin_base.html`, add a small script that reads the `_csrf_token`
cookie and configures HTMX to include it in all requests:

```html
<script>
    document.addEventListener('htmx:configRequest', function(evt) {
        const csrfToken = document.cookie
            .split('; ')
            .find(row => row.startsWith('_csrf_token='))
            ?.split('=')[1];
        if (csrfToken) {
            evt.detail.headers['X-CSRF-Token'] = csrfToken;
        }
    });
</script>
```

This single script in the admin base template covers all child forms — no need to add
`hx-headers` individually to each form element.

### CSRF token in login form
The login form (`POST /login`) does not need CSRF protection because the user is not
yet authenticated — there is no session to forge. CSRF protection applies only to
authenticated state-changing requests.

## References
- [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
- [OWASP Double-Submit Cookie Pattern](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#double-submit-cookie-pattern)
- [HTMX hx-headers attribute](https://htmx.org/attributes/hx-headers/)
- [HTMX htmx:configRequest event](https://htmx.org/events/#htmx:configRequest)
- [SameSite cookie attribute](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Set-Cookie#samesitesamesite-value)
