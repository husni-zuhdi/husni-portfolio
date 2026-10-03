# Keep content caching optional and process-local

Use an optional Moka in-memory cache for content reads instead of maintaining
Redis or Valkey infrastructure, which would add cost. Each running app instance
keeps its own cache and relies on TTL and local write invalidation rather than
shared cache state.
