# husni-portfolio

A server-rendered portfolio website showcasing blogs, talks, and tags. Built on Axum with Askama templates, Turso/SQLite for persistence, and Moka for in-memory caching. Admin operations are protected by JWT authentication.

## Language

**Blog**:
A content entry with a name, body (markdown), and associated tags. Blogs are the primary content unit of the portfolio.
_Avoid_: Post, article, entry

**Talk**:
A speaking engagement record with a name, date, and optional media/organisation links. Talks are displayed alongside blogs but have a distinct schema (no body, no tags).
_Avoid_: Presentation, event

**Tag**:
A label applied to one or more blogs. Tags enable filtering and categorisation. Each tag has a unique name and an integer id.
_Avoid_: Label, category

**BlogTagMapping**:
The many-to-many relationship between a Blog and a Tag. A mapping links one blog_id to one tag_id. Blogs can have multiple tags; tags can apply to multiple blogs.
_Avoid_: BlogTag, tag relation

**BlogMetadata**:
A lightweight projection of a Blog containing only id, name, filename, and tags — used for list views where the full body is unnecessary.
_Avoid_: BlogSummary, BlogPreview

**CommandStatus**:
An enum returned by write operations (Stored, Updated, Deleted, CacheInserted, CacheInvalidated) indicating the outcome of a persistence action.
_Avoid_: Result, Status

**Usecase**:
A module that owns business logic and orchestrates one entity's interactions with its storage adapters (database and optional cache). Each entity has one usecase. The usecase defines the interface callers use; repos are hidden behind it.
_Avoid_: Service, handler, controller

**Repo**:
A trait that defines storage operations for one entity (find, add, update, delete). Concrete adapters (TursoDatabase, InMemoryCache) implement these traits. The usecase depends on repo traits, never on concrete types.
_Avoid_: Repository, DAO, data access

**AppState**:
The Axum application state struct, threaded through every handler. Holds the config and all usecases.
_Avoid_: Context, state (use AppState when referring to the Axum state specifically)

**Cache**:
An in-memory store (Moka-backed) that mirrors a subset of database data to avoid repeated DB reads. Cache adapters implement the same display repo traits as database adapters, enabling the usecase to treat them uniformly.
_Avoid_: In-memory store, memoisation
