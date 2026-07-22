# Merge Display and Operation repo traits

Each entity currently defines separate traits for reads (BlogDisplayRepo) and writes (BlogOperationRepo). This split exists because the cache adapter only implements the Display trait plus cache-specific operations, while the DB adapter implements both. The split leaks into usecases, which hold two separate trait objects per backend.

We decided to merge these into one trait per backend per entity (e.g., `BlogRepo` with find + add + update + delete). The cache adapter implements a separate `BlogCacheRepo` trait with insert + invalidate + the same find methods. This means the usecase holds one DB trait object and one optional cache trait object, instead of two DB trait objects plus a third cache trait object.

**Considered Options:**

- _Keep Display/Operation split:_ More granular, but triples the number of trait objects per usecase (display_repo, operation_repo, cache_operation_repo). The handler still has to pick which field to call.
- _Merge into one trait per backend (chosen):_ Cleaner usecase interface. One `db` field, one `cache` field. The cache trait is a superset of display (it adds insert/invalidate).

**Consequences:**

- Existing repo implementations (TursoDatabase, InMemoryCache) will need to consolidate their trait impls into the merged trait.
- The usecase struct fields simplify from `blog_display_repo` + `blog_operation_repo` + `blog_cache_operation_repo` to `db: Box<dyn BlogRepo>` + `cache: Option<Box<dyn BlogCacheRepo>>`.
- Old tests on the individual traits become obsolete once tests at the usecase interface exist.
