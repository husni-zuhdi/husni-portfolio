# Repository guide for contributors and AI agents

Read this file first, then [`CONTEXT.md`](./CONTEXT.md) for domain terms. Use the
README for setup, [`specs/README.md`](./specs/README.md) for feature notes, and
[`docs/adr/README.md`](./docs/adr/README.md) for recorded decisions. Verify behavior
against the code when a spec and implementation differ.

## Project map

- `src/main.rs`, `src/config.rs`, `src/state.rs`, and `src/routes.rs` load
  configuration, construct application state, and connect HTTP routes.
- `src/handler/` contains HTTP request handling, including public pages and Admin
  area operations.
- `src/model/` contains domain and template data types.
- `src/usecase/` groups operations exposed to handlers.
- `src/repo/` defines data-access interfaces; `src/database/turso/` provides the
  SQLite and remote Turso adapter; `src/cache/inmemory/` provides in-process cache
  adapters.
- `templates/` contains Askama HTML templates. `statics/` contains CSS, JavaScript,
  and other static assets.
- Most Rust tests are colocated with their implementation under `#[cfg(test)]`.

The request path is generally `route → handler → usecase → repo interface →
database or cache adapter`. Keep changes local to the relevant module. Prefer a
small interface with behavior behind it; only add a seam when behavior actually
varies (one adapter is a hypothetical seam; two adapters make it real).

## Development and verification

The repository uses Rust 1.89.0. Taskfile tasks assume `sccache` is installed
because `RUSTC_WRAPPER` is set globally. See the README for the complete local
setup and optional tools needed for hot reload and CSS builds.

| Task | Purpose |
|---|---|
| `task run` | Run the app with file watching and rebuild CSS |
| `task test` | Run all-feature Rust tests serially |
| `task lint` | Run Clippy with warnings denied |
| `task fmt` | Format Rust files |
| `task audit` | Run `cargo-deny` checks |
| `task coverage` | Generate an HTML coverage report (requires `cargo-tarpaulin`) |

CI-equivalent Rust checks:

```sh
cargo test --all-features --locked -- --test-threads=1
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo deny check all
```

`cargo deny` is its own CLI; do not pass Cargo build flags such as `--locked` or
`--all-features` to it.

Keep tests serial when they mutate process-wide environment variables. Do not
weaken a check or skip an existing verification step to make a change pass; fix
the underlying issue or explain a concrete limitation.

## Configuration and data safety

- Use `env.example` as a template only. Replace placeholder secrets with local
  values; never commit `.env` or credentials.
- Do not read or print `.env`, `.release.env`, `service_account.json`,
  `release.service_account.json`, local database files, or secret values unless a
  task requires them and the user explicitly authorizes it.
- Do not use production credentials or databases for local tests.
- Database schema creation currently lives in `src/database/turso/mod.rs`; there
  is no active SQLx migration workflow.
- Administrator sign-in finds an existing user. No user-creation handler or
  documented account-provisioning procedure was found; do not invent one when
  documenting or testing Admin area access.

## Change guidance

- Preserve the existing Rust, Askama, and Taskfile patterns unless the task calls
  for a deliberate change.
- Update `CONTEXT.md` when a domain term or its meaning changes. Update a feature
  note when implemented behavior changes; label proposals and known limitations
  clearly.
- Read relevant ADRs before revisiting a technology choice. Record decisions in
  `docs/adr/` when they are hard to reverse, surprising without context, and the
  result of a real trade-off.
- Keep secrets, local databases, generated build output, and unrelated user work
  out of changes.
- For an architectural change, describe the module, its interface, depth, and seam;
  explain the leverage for callers and locality for maintainers. Tests should
  exercise the module through its interface where practical.
