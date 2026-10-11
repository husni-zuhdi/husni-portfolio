# Husni Portfolio

A server-rendered personal portfolio with public Blog and Talk pages and an Admin
area for managing content. See [`CONTEXT.md`](./CONTEXT.md) for domain terms and
[`AGENTS.md`](./AGENTS.md) for the module map and contributor/AI-agent guidance.

## Local development

### Prerequisites

- Rust 1.89.0 with `rustfmt` and `clippy`
- [Task](https://taskfile.dev/)
- [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) (for `task audit`)
- [sccache](https://github.com/mozilla/sccache) (Taskfile sets `RUSTC_WRAPPER=sccache`)
- For `task run`: [`cargo-watch`](https://github.com/watchee/cargo-watch) and the
  [Tailwind CSS CLI](https://tailwindcss.com/)
- For container tasks: Docker and `jq`

### Configure and run

```sh
cp env.example .env
```

Edit `.env` before running the app. At minimum, set `JWT_SECRET` to a private,
random value and check `SVC_ENDPOINT`, `SVC_PORT`, and `DATABASE_URL`. The example
selects local SQLite (`DATA_SOURCE=sqlite`) and enables the in-memory cache with a
one-hour TTL. Remove `CACHE_TYPE` to disable caching, or keep `CACHE_TTL` set when
the cache is enabled. Leave `SECRETS_BUCKET` and `SECRETS_OBJECT` blank for local
SQLite; set both only when loading secrets from Google Cloud Storage. Do not commit
`.env` or credentials.

```sh
task run
```

The app listens at `http://127.0.0.1:8080` with the example endpoint and port.
SQLite tables are created by the database adapter at startup. For remote Turso,
set `DATA_SOURCE=turso`, `DATABASE_URL` to the Turso database URL, and
`TURSO_AUTH_TOKEN` to a valid token.

Administrator sign-in only authenticates an existing user. This repository does
not currently document an account-provisioning procedure or provide a user
creation page.

### Common tasks

```sh
task test       # all-feature tests, run serially
task lint       # Clippy, warnings denied
task fmt        # format Rust files
task audit      # cargo-deny checks
task coverage  # HTML coverage report; requires cargo-tarpaulin
```

The corresponding CI checks are:

```sh
cargo test --all-features --locked -- --test-threads=1
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo deny check all
```

Most tests are inline in the Rust modules. Tests that mutate process environment
variables should remain serial. Follow
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for commit
messages.

## Configuration

`env.example` lists the supported settings. Important values:

| Variable | Purpose |
|---|---|
| `SVC_ENDPOINT`, `SVC_PORT` | Bind address and port; required when starting the app |
| `JWT_SECRET` | Signing secret used for Admin area authentication; set a private value |
| `DATA_SOURCE` | `sqlite` (default) or `turso` |
| `DATABASE_URL` | Local SQLite path or remote Turso URL |
| `TURSO_AUTH_TOKEN` | Required for a remote Turso database |
| `CACHE_TYPE`, `CACHE_TTL` | Optional in-memory cache and its TTL; TTL is required when enabled |
| `SECRETS_BUCKET`, `SECRETS_OBJECT` | Optional Google Cloud Storage object containing supported secrets |

When both GCS settings are configured, the app loads `JWT_SECRET`,
`DATABASE_URL`, and `TURSO_AUTH_TOKEN` from that object. See `src/config.rs` for
parsing behavior and [`specs/README.md`](./specs/README.md) for feature details.
The cost-driven GCS choice is recorded in
[`ADR-0006`](./docs/adr/0006-gcs-application-secrets.md).

## Docker Compose

The development Compose task expects `.env`. If that configuration loads secrets
from Google Cloud Storage, also provide Google application credentials in a file
named exactly `service_account.json` at the repository root; Compose mounts that
file at `/var/service_account.json`. Set `SVC_ENDPOINT=0.0.0.0` in `.env` while
using Compose so the app listens on the container interface; `127.0.0.1` is
appropriate when running the app directly on the host. Do not rename or commit a
credential file.

```sh
task docker-compose-up
task docker-compose-down
```

`task docker-build` builds the image locally. The GitHub release workflow builds
and pushes an image when a version tag is pushed; deployment environment variables
and cloud credentials must be configured in the deployment environment.

## Project documentation

- [`AGENTS.md`](./AGENTS.md) — contributor and AI-agent instructions
- [`CONTEXT.md`](./CONTEXT.md) — domain glossary
- [`specs/README.md`](./specs/README.md) — feature-note index, current behavior, and known limitations
- [`docs/adr/README.md`](./docs/adr/README.md) — architecture and technology decisions
- [`STORIES.md`](./STORIES.md) — historical work notes and backlog
