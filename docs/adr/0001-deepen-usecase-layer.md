# Deepen the usecase layer

The usecase layer is currently 100% pass-through delegation — each usecase wraps a repo trait object and forwards every call unchanged. This means the cache-then-DB orchestration, tag mapping coordination, and auth business logic that _should_ live in usecases is instead copy-pasted across 15+ handlers (~900 lines of identical boilerplate).

We decided to deepen each entity's usecase to absorb this logic. Each usecase will hold a DB adapter and an optional cache adapter behind its own interface. Handlers will call `blog_uc.find_blogs(params)` instead of manually locking mutexes, checking cache, falling back to DB, and backfilling.

**Considered Options:**

- _Keep pass-through usecases:_ Simple, but the duplication in handlers grows with every new feature. No place to test cache-then-DB interaction.
- _Move logic into handlers only:_ The current state — proven to cause duplication.
- _Deepen usecases (chosen):_ One implementation of cache-then-DB per entity. Handlers become thin. Testable through the usecase interface with mock adapters.

**Consequences:**

- Every handler that currently does cache-then-DB (15+) will be simplified to a single usecase call + render.
- Usecases become the primary test surface. Handler tests become optional (they just wire usecase → template).
- AppState shrinks — no more separate DB and Cache usecase fields per entity.
