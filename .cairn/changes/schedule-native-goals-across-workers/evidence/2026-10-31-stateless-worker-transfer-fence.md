# Stateless worker transfer fence (2026-10-31)

Scope: make a remote worker execute an action without holding the client's
coordinator state. The Buck2-like model has workers as stateless action
executors; the scheduler owns attempts and retries. Mantle's receiver previously
fenced output transfers against a job it had never seen unless client and worker
shared one coordinator state directory (true in the loopback production tests,
not true for separate hosts).

## Change

- `validate_current_production_attempt_if_known` treats an unknown job as "no
  local fence" and delegates to the strict check when the job exists.
- The receiver's generic transfer fence uses it, so an unknown job no longer
  blocks an output download.
- Workspace/lease paths keep the strict `validate_current_production_attempt`,
  because those mutate shared state; the upload path already passed no fence.
- A known job with a different attempt is still rejected, so a stale attempt
  cannot resurrect a job the worker does know.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle worker_transfer_fence` | 1 passed |
| `cargo test -p mantle --bin mantle remote_build::tests::` | 207 passed, 1 failed — the pre-existing `attenuated_admission_tests::denial_reason_codes_appear_only_at_debug_level` (reproduces on the parent commit, passes alone) |
| `cargo test -p mantle --test remote_transfer_production` | 12 passed |
| `cargo fmt --check` on `src/remote_build.rs` | exit 0 |

The production transfer suite was re-run after a full rebuild (the build cache
had to be cleared for disk space): `cargo test -p mantle --test
remote_transfer_production` reports **12 passed** after this change.

## Not claimed

- This does not prove separate-host execution; the qualification change owns
  independent-host acceptance.
- Skipping an unknown-job fence is safe only because the download path mutates
  no shared worker state; workspace and lease decisions remain strict.
