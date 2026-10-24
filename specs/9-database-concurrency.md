# Database Concurrency

**Status: Proposed (not implemented).** This note documents a plan agreed
after tracing a CI failure; nothing below describes current behavior except
where labeled "Today".

## Today (observed behavior)

- Every `TursoDatabase::new` run in `sqlite` mode opens `file:local.db`
  (the `Config::default()` URL), takes no WAL or busy-timeout settings, and
  executes 8 `CREATE TABLE IF NOT EXISTS` migrations plus profile seeding
  with `.expect()` on failure (`src/database/turso/mod.rs`).
- Every state-building test calls `state_factory(Config::default())`, so it
  touches the same shared `local.db`.
- The CI `nextest_unit_test` job runs `cargo nextest run` (parallel,
  process-per-test). Concurrent processes taking the SQLite writer lock on
  `local.db` abort with `SqliteFailure(5, "database is locked")` through the
  unconditional `.expect("Failed to migrate tables.")` at
  `src/database/turso/mod.rs` (migrations loop). The failure lands on an
  arbitrary test because construction order is nondeterministic.

## Failure trace (2026-10-24 CI run)

`handler::admin::profile::operations::test::test_post_add_admin_profile_section_unauthorized`
panicked at `src/database/turso/mod.rs` (`Failed to migrate tables.`) with
`database is locked`. The test itself only exercises an unauthorized branch;
it failed because `state_factory` builds the full `AppState` (including
migrations) before the auth check. Under nextest, several test processes ran
their migrations against the same `local.db` simultaneously.

The classic `unit_test` CI job (`cargo test -- --test-threads=1`) passed;
the repo's documented CI-equivalent runs tests serially, which is why this
race was latent until the nextest job grew enough DB-writing tests
(profile migrations plus seeding widened the write window).

## Proposed changes (Grilled: decision 2 of 3; user prefers maximal concurrency)

### 1. DB hardening in `TursoDatabase::new` (sqlite branch)

After the connection is created, before migrations, and only for file-based
URLs (skip `:memory:` and remote Turso):

- `PRAGMA journal_mode=WAL;` via `conn.execute`
- `conn.busy_timeout(Duration::from_secs(15))`

Verified available in libsql 0.10.0: `Connection::busy_timeout` (backed by
native `sqlite3_busy_timeout` for local connections) and the `journal_mode`
PRAGMA are whitelisted in the libsql statement parser. WAL allows readers to
proceed while a single writer commits; the busy timeout absorbs writer-writer
contention instead of panicking. This also benefits production `sqlite`
deployments.

### 2. Per-test database isolation (the concurrency unlock)

- Add a `#[cfg(test)]` helper on `Config` (working name `default_test()`)
  that clones `Config::default()` but sets
  `secrets.database_url = ":memory:"`. `Builder::new_local(":memory:")` is
  the officially documented in-memory mode for libsql.
- Swap every state-building test's `state_factory(Config::default())` to the
  helper. Pure template-rendering tests and config-string tests keep
  `Config::default()`.

Why `:memory:` *per state-construction* rather than a unique temp file per
process: classic `cargo test` runs many tests concurrently inside one
process, so a per-process file would just move the writer lock contention
in-process. In-memory-per-call is fully parallel in both runners (nextest's
process-per-test and cargo-test's in-process threads). Verified safe: no
test builds two states and expects cross-state persistence; seeding runs
per-instance so every state starts from the seeded profile snapshot.

### 3. Housekeeping

- Add `local.db-wal` and `local.db-shm` to `.gitignore` (WAL sidecars
  appear next to `local.db` in dev runs of `task run`).
- CI workflows need no changes; nextest stays fully parallel.

## Verification plan

1. Serial sanity: `cargo test --all-features --locked -- --test-threads=1`.
2. Parallel proof: `cargo test --all-features --locked` without the serial
   flag (same owner-writer race surface as nextest).
3. `cargo fmt --check` and `cargo clippy --locked -- -D warnings`.

## Accepted tradeoffs

- Tests stop exercising migration-on-a-shared-file behavior; production
  still runs the identical code path on `local.db`.
- `:memory:` DBs are transient; nothing persists across test states
  (checked: nothing relies on that).
