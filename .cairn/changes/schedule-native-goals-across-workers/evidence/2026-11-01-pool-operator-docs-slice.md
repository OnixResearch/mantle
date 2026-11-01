# Pool operator documentation slice (2026-11-01)

Scope: document the pool operator contract the implementation enforces, and
index it from the README.

## What changed

- `docs/remote-pool.md`: CLI flags and clap conflicts, the Nickel pool schema
  shape, binding and ticket-descriptor requirements, admission rules (endpoint
  and credential uniqueness, key material uniqueness per endpoint, bounded
  session, inventory requirement for concurrent sessions, measured transport
  program identity), execution and output admission, failure settlement, and
  explicit non-claims.
- `README.md`: links the new page from the documentation list.

The page states only behavior exercised or read from the current
implementation; it adds no runtime or release claim.

## Verification

Prose-only change: source review against `src/main.rs` (clap conflicts),
`src/remote_build/pool_config.rs`, `src/remote_build/pool_factory.rs`, and
`crates/crunch-remote-core/src/pool.rs` (admission bounds and uniqueness rules).
No test asserts prose.

## Non-claims

- `ssh-stdio` and `p2p` transports are documented as admitted but unproven.
- Cancellation, stale fences, duplicate results, capacity shrink, and
  worker-death-mid-exchange remain uncovered.
