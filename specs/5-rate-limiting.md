# Login rate limiting

**Status: Implemented with a reverse-proxy limitation.** This note describes the
current route and configuration.

## Behavior

- Only `POST /login` is rate limited; `GET /login` is not.
- The limiter uses `tower-governor` and its default peer-IP key extractor.
- `RATE_LIMIT_BURST_SIZE` defaults to `10` and sets the burst size.
- `RATE_LIMIT_REPLENISH_PERIOD_SECOND` defaults to `60` and configures the
  replenish period in seconds.
- Rate limit headers are enabled. The router starts a background Tokio task that
  calls `retain_recent()` every 60 seconds to clean up idle limiter entries.
- Rate-limited requests are rejected by the governor middleware (429 response).

## Known limitation: reverse proxies

The current peer-IP extractor keys requests by the socket peer. Behind Cloud Run
or another reverse proxy, that peer can be the load balancer rather than the
original client, causing unrelated clients to share a rate-limit bucket. The
current implementation does not read proxy-forwarded IP headers. See the tracking
note in `STORIES.md` before changing the extractor; forwarded headers must only be
trusted when the deployment proxy is configured to set them safely.

## Main paths

- Rate-limited `POST /login` route and limiter cleanup task: `src/routes.rs`
- Peer address supplied to the router: `src/main.rs`
- Environment parsing and defaults: `src/config.rs`
- Example settings: `env.example`

Configuration tests live in `src/config.rs`. The workflow test suite runs Rust
tests serially because other tests mutate process-wide environment variables.

## References

- [tower-governor](https://docs.rs/tower_governor/latest/tower_governor/)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
