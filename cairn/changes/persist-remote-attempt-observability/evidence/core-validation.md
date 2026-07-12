# Immutable remote attempt log functional-core validation

Date: 2026-07-12

Change: `persist-remote-attempt-observability`

Requirements:

- `r[remote_builds.immutable_attempt_log_segments]`
- `r[remote_builds.pure_log_cursor_kernel]`

## Question

Does the partial slice provide a deterministic bounded functional core for immutable fenced attempt-log identities and decisions without claiming coordinator persistence, restart integration, telemetry, or production resumable-transfer integration?

## Inspected evidence

### Implementation

`crates/crunch-build/src/distributed/remote_attempt_log.rs` adds:

- named hard/default bounds and a validated `RemoteAttemptLogPolicy`;
- versioned record, segment, segment-reference, manifest, policy-identity, and truncation-anchor DTOs;
- BLAKE3 payload, record, segment, policy, manifest, and anchor identities;
- fenced current-attempt scope/phase checks;
- deterministic redaction/control escaping and bounded truncation;
- pure record/segment/manifest sealing and validation;
- append, duplicate idempotency, event conflict, exact sequence/cursor, previous-record, and previous-segment decisions;
- retained-chain validation, cursor classification, bounded replay, and whole-segment retention planning;
- explicit truncation anchors binding dropped cursor range, segment/record/byte counts, dropped tails, prior heads, policy identity, and previous-anchor identity;
- table, property, positive, negative, and `cfg(kani)` harnesses.

`crates/crunch-build/src/distributed.rs` exports the new sibling core. The module contains no filesystem, environment, process, clock, async, Tokio, or rendering calls.

### Before-change baseline

Pueue task 117:

```text
TMPDIR=/tmp/mantle-agent-observability-baseline/tmp \
CARGO_TARGET_DIR=/tmp/mantle-agent-observability-baseline/target \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
nix develop -c cargo test -p crunch-build --lib remote_attempt

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 523 filtered out; finished in 0.01s
```

Pueue task 122:

```text
TMPDIR=/tmp/mantle-agent-observability-baseline/tmp \
CARGO_TARGET_DIR=/tmp/mantle-agent-observability-baseline/target \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
nix develop -c cargo test -p mantle --bin mantle coordinator_log_replay_stays_bounded_for_slow_subscribers

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1396 filtered out; finished in 0.00s
```

### After-change focused and regression checks

Pueue task 278 ran the focused immutable-log tests in the isolated after-change target:

```text
TMPDIR=/tmp/mantle-agent-observability-after/tmp \
CARGO_TARGET_DIR=/tmp/mantle-agent-observability-after/target \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
nix develop -c cargo test -p crunch-build --lib remote_attempt_log

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 538 filtered out; finished in 0.07s
```

Pueue task 313 ran `cargo fmt --check -p crunch-build`, the full owning-crate library suite, and the legacy coordinator replay regression in the isolated after-change target. The chained command completed successfully. Pueue task 315 re-ran the owning-crate suite separately so its terminal result was captured:

```text
test result: ok. 551 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
```

The final leg of task 313 captured the unchanged mutable replay regression:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1396 filtered out; finished in 0.00s
```

### Kani and Clippy boundaries

Pueue task 304 showed that Kani is unavailable in the checked development shell:

```text
error: no such command: `kani`
```

The `cfg(kani)` arithmetic and cursor-classification harnesses are present, but their execution is **not claimed**.

A focused `cargo clippy -p crunch-build --lib --no-deps -- -D warnings` run produced no diagnostics for `remote_attempt_log.rs` after its two local findings were repaired. The package command remains nonzero on nine unchanged diagnostics outside this slice: three lifetime findings in `dynamic_plan.rs`, four large-error findings in `network_policy.rs`, and one large-enum plus one type-complexity finding in `worker.rs`. This evidence does not claim package-wide Clippy success.

### Cairn partial-slice validation

Pueue task 322 ran the native Cairn validation and all three requested advisory gates after the task/evidence update. The chained command completed successfully.

```text
cairn validate: valid=true, changes=7, specs_validated=34, issues=[]
proposal gate: verdict=PASS, valid=true, issues=[], receipt_hash=b03c07682d955739714517226e7e47cfc9509099e788985da830ab9d8a843825
design gate: verdict=PASS, valid=true, issues=[], receipt_hash=97e37e2c6d918cb10e72797771707b8f886869fced117e54ca2368d359bbb7db
tasks gate: verdict=PASS, valid=true, issues=[], receipt_hash=db7551ac56b45e3a43bb0ddf90def9bb6a7e9f9a3561525cb0b9d938b47524c8
```

These advisory gates validate the package shape and recorded partial status. They do not make unchecked implementation or verification tasks complete.

## Decision

The immutable attempt-log DTO and pure-decision slice is proven by the current Rust tests and formatting check. It is sufficient to check the inventory, DTO, pure-kernel, and core test tasks only.

The evidence does **not** prove immutable filesystem persistence, atomic manifest advancement, restart replay, deletion ordering, replacement of `RemoteCoordinatorState::logs`, remote protocol integration, telemetry, trace propagation, Prometheus/OTLP exporters, or production resumable-transfer instrumentation.

## Owner

The active Cairn change `persist-remote-attempt-observability` owns the unchecked shell and integration work. The active `complete-resumable-remote-cas-transfer` change owns the production transfer dependency.

## Next action

Implement a thin bounded shell only after defining segment/manifest paths and crash ordering: publish and fsync immutable content first, atomically advance/fsync the manifest second, update coordinator summaries third, and delete retention candidates only after the durable anchor transition. Then add restart and tamper fixtures before checking shell tasks.
