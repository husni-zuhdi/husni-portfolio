# GitHub Actions

**Status: Implemented.** This note describes the current workflow triggers and
checks. The YAML files in `.github/workflows/` are authoritative.

## Workflows

| Workflow | Trigger | Checks or action |
|---|---|---|
| `rust-ci.yml` | Pull requests and non-`main` branch pushes affecting Rust manifests, Rust source, Askama templates, CI/task configuration, or `env.example` | Serial Cargo tests, Nextest, formatting, Clippy, and coverage |
| `rust-audit.yml` | Weekly schedule; pull requests and non-`main` pushes affecting Cargo dependencies, `deny.toml`, or the audit workflow | `cargo-deny` dependency/license/advisory checks |
| `rust-release.yml` | Pushes to `main` | Runs release-plz to open/update a release pull request |
| `rust-push-build.yml` | Any pushed tag | Builds and pushes the Docker image to Google Container Registry |

The Rust CI and audit pull-request triggers are top-level `on.pull_request`
entries. Their path filters mean unrelated documentation-only changes do not run
those workflows.

## Release notes

The release workflow is configured with `changelog_update = false` in
`release-plz.toml`; it does not update `CHANGELOG.md` automatically. Keep the
changelog aligned with tagged releases and unreleased behavior changes.

## Verification

The local CI-equivalent Rust checks are documented in the repository root
[`AGENTS.md`](../AGENTS.md). Test commands use serial execution because some
configuration tests mutate process-wide environment variables.
