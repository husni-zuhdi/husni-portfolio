# In-memory caching

**Status: Implemented.** This note describes the current code, including a known
startup failure mode. The database remains the source of persisted content.

Decision: [ADR-0007](../docs/adr/0007-process-local-moka-cache.md).

## Behavior

- Caching is disabled unless `CACHE_TYPE` is set to `InMemory` (the parser also
  accepts `inmemory`).
- `CACHE_TTL` is a duration in seconds and is required when caching is enabled.
  `src/state.rs` currently unwraps this setting during startup, so omitting it
  causes a panic.
- The Moka in-memory adapter caches Blogs, Talks, Tags, and Blog-tag mappings.
  It is local to one running process; separate app instances do not share cache
  entries.
- Public reads can use cached records. Content changes made through the Admin area
  update or invalidate the corresponding cache entries after database changes.

## Configuration

| Variable | Meaning |
|---|---|
| `CACHE_TYPE` | Optional; `InMemory` enables the in-process cache. Any other value is treated as disabled. |
| `CACHE_TTL` | Optional integer number of seconds; required when `CACHE_TYPE` enables caching. |

Example:

```dotenv
CACHE_TYPE=InMemory
CACHE_TTL=3600
```

## Main paths

- Cache construction: `src/state.rs`
- Startup prefill: per-entity `prefill` verbs (`src/usecase/{blogs,tags,talks,profile}.rs`)
- Cache reads and updates/invalidation: inside the per-entity usecase verbs (`src/usecase/`); the usecases call the adapters in `src/cache/inmemory/`
- Cache data-access interfaces: `src/repo/`
- Configuration parsing: `src/config.rs`
- Read-through rules and all-or-nothing list reads: `specs/10-deep-usecases.md`

## Known limitation

When `CACHE_TYPE` enables the cache and `CACHE_TTL` is absent, application startup
panics at `src/state.rs` while constructing the adapter. Set both variables or
disable caching. This failure mode is tracked in `STORIES.md`.
