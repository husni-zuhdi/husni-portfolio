# Protect Admin area writes with double-submit CSRF

Validate a CSRF token from a browser-readable cookie against a request header on
state-changing Admin area requests, including HTMX writes. This protects
cookie-authenticated requests without adding a heavy session-management crate;
`SameSite=Strict` is defense-in-depth, as recorded in the original CSRF design
notes.
