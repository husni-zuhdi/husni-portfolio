# Architecture Decision Records

These records capture deliberate technology and architecture choices, their
rationale, and scope. Read a relevant record before proposing to replace a choice.

1. [Use libsql for SQLite and Turso](./0001-libsql-for-sqlite-and-turso.md)
2. [Use the database as the runtime source for Blogs](./0002-database-as-blog-runtime-source.md)
3. [Use password sign-in with JWT cookies](./0003-password-and-jwt-authentication.md)
4. [Protect Admin area writes with double-submit CSRF](./0004-double-submit-csrf-protection.md)
5. [Allow authored HTML and sanitize Blog output](./0005-sanitize-authored-blog-html.md)
6. [Use Google Cloud Storage for application secrets](./0006-gcs-application-secrets.md)
7. [Keep content caching optional and process-local](./0007-process-local-moka-cache.md)
8. [Use Axum for HTTP handling](./0008-axum-http-framework.md)
9. [Use Askama for HTML templates](./0009-askama-templates.md)
10. [Host the portfolio on Google Cloud Run](./0010-cloud-run-hosting.md)
