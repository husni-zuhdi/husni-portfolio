# Rate Limiting

## Goals
Prevent brute-force attacks on the login endpoint (`POST /login`) by limiting the number
of login attempts per IP address within a given time window. The current implementation
has no throttling, allowing unlimited login attempts at network speed.

## Criterias
- Rate limiting is applied **only** to `POST /login` (not to all routes)
- The rate limit configuration is configurable via environment variables
- Rate-limited requests receive a `429 Too Many Requests` response with `Retry-After` header
- Rate limit headers (`x-ratelimit-limit`, `x-ratelimit-remaining`, `x-ratelimit-after`)
  are included in responses
- The rate limiter uses the client's IP address as the key (with proxy-aware header
  extraction for Cloud Run deployment)
- Idle IP buckets are periodically cleaned up to prevent unbounded memory growth
- The `axumserve` startup uses `into_make_service_with_connect_info::<SocketAddr>()`
  to provide peer IP to the rate limiter

## Usage
[tower-governor](https://github.com/benwis/tower-governor) is a Tower middleware that
wraps the [governor](https://github.com/antifuchs/governor) rate-limiting crate. It uses
the Generic Cell Rate Algorithm (GCRA) — a smooth, allocation-free token bucket
implementation. Unlike fixed-window rate limiters, GCRA has no boundary burst issues.

Add `tower-governor` to `Cargo.toml`:
```toml
[dependencies]
tower-governor = { version = "0.8", features = ["axum"] }
```

### Key extractor
Use `SmartIpKeyExtractor` instead of the default `PeerIpKeyExtractor`. Behind Cloud Run
(and most reverse proxies), the peer IP is the load balancer's IP. `SmartIpKeyExtractor`
checks `X-Forwarded-For`, `X-Real-IP`, and `Forwarded` headers in order, falling back
to the peer IP. This is correct for the Cloud Run deployment.

### Configuration via environment variables
Follow the existing pattern (like `CACHE_TYPE`, `CACHE_TTL`):

Draft of envars:
- `RATE_LIMIT_BURST_SIZE`
    - Number (u32)
    - Maximum number of requests allowed in a burst before throttling begins
    - Default: `10`
- `RATE_LIMIT_REPLENISH_PERIOD_SECOND`
    - Number (u64)
    - Interval in seconds after which one token is replenished in the bucket
    - Default: `60` (one attempt per minute after burst is exhausted)

### Preset: GovernorConfig::secure()
The `tower-governor` crate provides a `GovernorConfig::secure()` preset: burst of 2,
replenish 1 per 4 seconds. This is designed specifically for login endpoints. Our
configurable env vars allow tuning beyond this preset.

## Flow

### Normal login (within rate limit)

```mermaid
sequenceDiagram
    participant U as User (Browser)
    participant RL as Rate Limiter (Governor)
    participant H as Login Handler

    U->>RL: POST /login
    RL->>RL: Check IP bucket (key: SmartIp)
    RL->>RL: Token available? Yes (decrement)
    RL->>H: Forward request
    H->>H: Validate credentials
    H-->>U: 200 OK (Set-Cookie: token=..., _csrf_token=...)
```

### Rate-limited login (burst exhausted)

```mermaid
sequenceDiagram
    participant U as Attacker
    participant RL as Rate Limiter (Governor)
    participant H as Login Handler

    U->>RL: POST /login (attempt 6)
    RL->>RL: Check IP bucket
    RL->>RL: Token available? No (bucket empty)
    RL-->>U: 429 Too Many Requests
    Note over U: Headers: Retry-After: 45, x-ratelimit-after: 45
    U->>RL: POST /login (attempt 7)
    RL-->>U: 429 Too Many Requests
    Note over U: Must wait for token replenishment
```

### Bucket cleanup

```mermaid
sequenceDiagram
    participant T as Tokio Task
    participant L as Governor Limiter

    loop Every 60 seconds
        T->>L: retain_recent()
        L->>L: Remove IP buckets with no recent activity
    end
```

## Implementation locations

| File | Change |
|---|---|
| `Cargo.toml` | Add `tower-governor = { version = "0.8", features = ["axum"] }` |
| `src/config.rs` | Add `rate_limit_burst_size: u32` and `rate_limit_replenish_period: u64` fields to `Config`, read from env vars with defaults |
| `src/routes.rs` | Create `GovernorConfig` with `SmartIpKeyExtractor`, attach `GovernorLayer` only to the `POST /login` route |
| `src/main.rs` | Change `axum::serve(listener, app)` to `axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())` |
| `src/main.rs` | Spawn background task for `limiter.retain_recent()` every 60 seconds |

### Route-level layer attachment
The rate limiter must be scoped to `POST /login` only. This is done by applying the
layer to a sub-router:

```rust
// In routes.rs
fn login_route(rate_limit_config: Arc<GovernorConfig>) -> Router<AppState> {
    Router::new()
        .route("/login", post(ao::post_login))
        .layer(GovernorLayer::new(rate_limit_config))
}

// main_route merges it
.route("/login", get(ad::get_login))  // GET is not rate-limited
.nest("/login", login_route(rate_limit_config))
```

Alternatively, apply the layer directly to the POST handler using `axum::routing::post`
with a `.layer()` on that specific route.

### GovernorConfig construction
```rust
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;

let governor_conf = GovernorConfigBuilder::default()
    .per_second(config.rate_limit_replenish_period)
    .burst_size(config.rate_limit_burst_size)
    .use_headers()
    .finish()
    .unwrap();
```

### Startup change
```rust
// main.rs — current
axum::serve(listener, app).await.unwrap();

// main.rs — required for SmartIpKeyExtractor to access peer SocketAddr
axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
```

## Testing
Config parsing tests live in `src/config.rs` under `#[cfg(test)] mod test`:
- `test_default` — asserts `rate_limit_burst_size = 10`, `rate_limit_replenish_period = 60`
- `test_from_envar_without_optionals` — asserts defaults apply when env vars are absent
- `test_from_envar_with_optionals` — asserts custom values are parsed from env vars

## References
- [tower-governor crate](https://github.com/benwis/tower-governor)
- [tower-governor docs.rs](https://docs.rs/tower-governor/latest/tower_governor/)
- [governor crate (rate limiting core)](https://github.com/antifuchs/governor)
- [SmartIpKeyExtractor docs](https://docs.rs/tower-governor/latest/tower_governor/key_extractor/struct.SmartIpKeyExtractor.html)
- [OWASP Brute Force Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Brute_Force_Prevention_Cheat_Sheet.html)
- [GCRA algorithm explanation](https://github.com/antifuchs/governor/blob/master/GLOSSARY.md)
