# Deep per-entity usecases

**Status: Implemented.** This note documents the usecase-layer refactor: HTTP
handlers call one deep usecase per content entity, and that usecase owns the
database repos, the optional in-process cache, and all read-write choreography
behind its verb interface.

## Today (before the refactor)

The usecase layer was 100% forwarding: `BlogDBUseCase`, `TagCacheUseCase` and
their siblings re-exported repo traits with public fields, added no behavior,
and had no tests (~625 lines across `src/usecase/`). Handlers read those
public fields and hand-copied the read-through and invalidation rules around
30 call sites, so cache behavior differed between pages (some reads refilled,
some did not) and list reads could return partial results when cache entries
expired per key.

## Current behavior

### One usecase per content entity

- `BlogUseCase` (`src/usecase/blogs.rs`): Blog reads/writes plus the Blog tag
  choreography.
- `TagUseCase` (`src/usecase/tags.rs`): Tag reads/writes plus the shared
  `ensure_tag_names` verb the Blog writes depend on.
- `BlogTagMappingUseCase` (`src/usecase/blog_tag_mappings.rs`): Blog tag
  mapping reads/writes. Consumed only by the Blog usecase; not exposed on
  `AppState`.
- `TalkUseCase` (`src/usecase/talks.rs`) and `ProfileUseCase`
  (`src/usecase/profile.rs`): Talk and Profile (sections + meta) reads and
  writes.
- Auth stays as-is (`AuthDBUseCase`); the auth feature is out of scope and
  remains a candidate for later cleanup.

Handlers hold `Arc<Mutex<XUseCase>>` clones from `AppState`
(`blog_usecase`, `tag_usecase`, `talk_usecase`, `profile_usecase`) and call
verbs only. Repo trait objects are private fields.

### Read-through rules

- Detail reads (one Blog/Tag/Talk, one Profile section) hit the cache first,
  fall through to the database, and refill the cache on a database hit.
- List reads (Blogs, Tags, Talks) are all-or-nothing: the cache adapter keeps
  an id index (`blogs_index`, `tags_index`, `talks_index` inside
  `InMemoryCache`) of ids known to exist. An indexed id missing from the
  cache makes the whole read a miss (falls to the database and refills); an
  unindexed id is a gap — content that does not exist in the database — not
  a miss. This removes the stale/partial list bug.
- `TagUseCase.search_tags` is database-truth by contract: the cache only
  holds tags present from earlier list reads, so it cannot prove that an
  uncached id does not match the LIKE query.
- `BlogTagMappingUseCase` reads resolve to the database directly (mapping
  truth must not race with Blog writes); its write verbs own cache
  insert/invalidation.
- `ProfileUseCase` caches the page as one unit (`ProfilePage` = sections +
  meta). Writes drop and rebuild the whole page cache; plain reads rebuild
  the page when the cached unit is absent.

### Write verbs own their cache choreography

Each `add`/`update`/`delete` verb applies the database change, then refreshes
or drops the affected cache entries inside the usecase. There is no
per-callsite "is cache enabled" decision: cache presence is resolved at
construction (`with_cache`) and writes are wired with or without a cache.
Blog writes additionally keep Tags and Blog tag mappings in sync (the former
via `TagUseCase::ensure_tag_names`); every Profile write rebuilds the page
cache.

### Verb truth conventions

- Repo methods keep the layer's `Option` convention (`None` = failure).
  Composite verbs collapse a failing correlated step to `None`; handlers map
  `None` to 404 (add/update) or 500 (delete) as they did per-step before.
- Startup prefill lives on the usecases (`prefill` per entity, called by
  `src/state.rs`); the old shared `prefill_inmemory_cache` walker is gone.

## Testing

Each usecase carries colocated `#[cfg(test)]` tests against the real stack
(SQLite via `TursoDatabase::new("sqlite", "file:<tmp>", None)` plus the real
`InMemoryCache`), serialized, with unique temp database files per test.
The cache adapters add all-or-nothing regression tests (e.g.
`test_find_blogs_all_or_nothing_on_partial_cache`).

## Known consequences

- A write that succeeds in the database but fails in a correlated step keeps
  the committed change and surfaces one error page (no per-step retry flows
  existed before, so nothing was lost).
- Tag deletion does not clean orphan Blog-tag mapping rows — unchanged legacy
  behavior (no foreign-key cascade); candidate for the repo-hardening pass.
- 70+ `.expect()` panics remain in the database adapters (None vs error
  modes); a separate hardening candidate owns that cleanup.

## Main paths

- Usecases: `src/usecase/{blogs,tags,blog_tag_mappings,talks,profile}.rs`
- Wiring: `src/state.rs` (`create_db_usecases`, `state_factory`),
  `src/model/axum.rs` (`AppState`)
- Cache adapters + indexes: `src/cache/inmemory/`

## References

- ADR-0007 (process-local Moka cache)
- `specs/2-caching.md` for configuration
