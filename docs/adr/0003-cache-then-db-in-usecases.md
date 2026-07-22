# Cache-then-DB in usecases

Every data-fetching handler repeats the same pattern: lock cache mutex → check cache → fall back to DB → backfill cache → render. This appears 15+ times across public and admin handlers, totaling ~900 lines of near-identical code. The pattern _is_ business logic — it determines where data comes from and how the cache stays warm.

We decided to move cache-then-DB into the usecase. The usecase's `find` method checks the cache adapter first, falls back to the DB adapter on miss, and backfills the cache. On writes, the usecase invalidates + re-inserts the cache entry. Handlers call one method; the usecase decides.

**Considered Options:**

- _Axum middleware:_ Would intercept all requests uniformly, but the cache-then-DB pattern is entity-specific (blogs cache blogs, tags cache tags). A generic middleware can't know which cache to check.
- _Cache decorator (adapter pattern):_ A `CachedBlogRepo` wraps a `BlogRepo` and adds caching transparently. Clean, but the decorator would need access to both the cache and DB repos — it becomes the usecase by another name.
- _In the usecase (chosen):_ The usecase already owns both adapters. The logic lives where it naturally belongs. The interface is the test surface.

**Consequences:**

- Handlers shrink from ~60-100 lines to ~10 lines (call usecase, render template).
- Cache-then-DB interaction is tested once in the usecase, not duplicated across 15 handler tests.
- Cache backfill and invalidation logic is localised to the usecase — fixing a cache bug means editing one file.
