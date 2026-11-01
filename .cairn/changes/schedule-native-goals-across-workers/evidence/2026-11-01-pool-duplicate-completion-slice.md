# Duplicate-completion refusal slice (2026-11-01)

Scope: a second completion report for an already-finished production attempt
must be refused and must not change the job or resurrect a resource lease.

## What changed

- `src/remote_build.rs`: the existing real-signed stdio successor test now
  applies a duplicate completion after the durable reload and asserts:
  the call fails with `terminal-attempt`, `already-applied`, or
  `current-attempt-completed`; the job record is byte-identical to the finished
  one; no resource lease exists.

This is the coordinator rule the pool executor relies on when a worker result is
redelivered or a completion is replayed; the pool transaction lock and revision
retry do not turn a duplicate into a second publication.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle -- real_signed_stdio_stale_result_is_rejected_before_import_and_successor_imports` | `test result: ok. 1 passed` |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p mantle` | exit 0 |

## Non-claims

- The test uses the fixture local executor, not a live `remote serve` worker.
- No stale-fence replacement, lease revocation, or cross-process duplicate
  completion is exercised.
