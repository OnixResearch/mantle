# Production remote-attempt observability validation

Date: 2026-07-12

Change: `persist-remote-attempt-observability`

Requirements:

- `r[remote_builds.immutable_attempt_log_segments]`
- `r[remote_builds.diagnostic_trace_context]`
- `r[operator_diagnostics.remote_execution_telemetry]`
- `r[operator_diagnostics.telemetry_exporter_isolation]`

## Question

Does the production stdio remote-build path persist bounded lifecycle diagnostics, execute and report a real bounded remote-root priority decision, propagate W3C trace context without authority, isolate exporter/log failures from build truth, and expose contracted health without fabricating publication events?

## Inspected evidence

Implementation inspection established these production boundaries:

- `src/main.rs::run_remote_build_dispatches_async()` snapshots the evaluated remote roots once, delegates ranking to the existing scheduling kernel, executes the resulting input order, records one `priority-selected` event at that selection boundary, and then records route, queue admission, assignment, current-fence acceptance, execution/protocol failure, and output-admission acceptance or rejection only after the corresponding transition.
- `src/remote_build.rs::plan_remote_root_priority()` is a pure bounded planner over `SchedulingPolicy`, `ReadyGoalFacts`, `KnownGraphPressure`, `rank_ready_goals()`, and `priority_decision_evidence()`. It rejects empty, oversized, duplicate-index, duplicate-identity, and malformed-identity snapshots deterministically.
- Remote-root identities are BLAKE3 values derived from stable request facts. The planner uses ordinary/default preference, a single ranking pass, structural-fallback history, and explicit zero known-graph pressure because this seam has no honest critical-path or blocked-root facts. Raw labels, store paths, provider ids, and goal ids do not become metric labels.
- `src/remote_build.rs::run_remote_production_client_protocol()` records execution start/completion and actual transfer demand, credit, resume, cutoff, completion, and fallback facts.
- Intentional checkpoint interruption is classified as rejected transfer cutoff, not execution failure. Generic protocol/execution failures use `execution-failed`; output-admission failures use `output-rejected`.
- No publication event is emitted because this path has no publisher adapter or successful publication transition.
- `src/remote_trace_context.rs` accepts only bounded W3C `traceparent`/`tracestate`, exports digest-only health, and keeps trace data in a negotiated diagnostic frame outside concrete request, auth, coordinator key, assignment/fence, scheduling, and output-admission inputs.
- `src/remote_attempt_log_store.rs` persists immutable segment/manifest/anchor state. Coordinator state retains only the bounded `immutable_log` control summary; stale-fence diagnostics use a separate derived scope and do not rewrite the superseded attempt manifest.
- `src/remote_telemetry_export.rs` consumes canonical low-cardinality descriptors through disabled-by-default bounded Prometheus/OTLP shells and returns health rather than changing build truth.
- `schemas/machine-contracts/build-json-report.schema.json` owns the closed optional telemetry, priority-decision, and observability projection. The generated Nickel contract, negative fixture, inventory ownership, and Rust producer parity move together.

Current validation used the dedicated `/tmp/mantle-observability-final-target`, nightly Rust, clang, a live mold 2.40.4 wrapper, explicit OpenSSL pkg-config, disabled incremental compilation, test debug info disabled, and the executable statically linked sandbox shell at `/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`.

### Recovery integrity and compile

Pueue task 2018 canonically formatted the independently replayed source and proved it is byte-for-byte identical to `src/remote_build.rs` (`cmp_exit=0`). The replay contains all 49 successful pre-truncation edit calls and 105 exact replacements; no conversation text is part of this evidence.

Pueue task 2019 compiled the recovered production binary:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 07s
```

### Focused remote-build and production checks

Pueue task 2031 ran the complete `remote_build::tests` module, including the one-shot priority planner, its empty/duplicate/oversized negative table, trace-authority invariance, lifecycle mapping, transfer cutoff/resume, stale-fence isolation, immutable payload, and bounded metric tests:

```text
running 118 tests
test result: ok. 118 passed; 0 failed; 0 ignored; 0 measured; 1339 filtered out; finished in 1.02s
```

Pueue task 2033 ran the complete positive/negative multi-process production suite:

```text
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.85s
```

The process suite proves input/output restart, durable resume, ticket quota rejection, Prometheus textfile success, valid trace propagation, malformed/oversized trace dropping, OTLP collector outage isolation, immutable-log and exporter health, unchanged successful output bytes, and production ordering beginning with `priority-selected`, `route-selected`, `queue-admitted`, `worker-assigned`, and `fence-accepted`. It also asserts the report carries exactly one priority decision for the one-shot root snapshot, with the real competing-root count and zero critical-path/work/blocked-root facts under structural fallback.

Pueue task 2147 ran all four focused shell/config filters. Its status and preserved combined log were inspected directly:

```text
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1449 filtered out; finished in 0.01s
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1450 filtered out; finished in 0.00s
running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 1441 filtered out; finished in 0.04s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1454 filtered out; finished in 0.00s
```

These are, in order, immutable-log storage, exporter isolation, typed remote-farm configuration, and bounded trace-context tests.

Pueue task 2035 ran the complete `crunch-build` library regression suite:

```text
test result: ok. 555 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
```

### Closed machine contract

Pueue task 2110 ran generation, checker self-test, strict freshness validation, and the five targeted Rust producer-parity tests. Its status and preserved log were inspected directly and remain available in the shared daemon:

```text
machine schema contract generation: PASS (16 contracted, 45 classified)
machine schema contract self-test: PASS
machine schema contract check: PASS (16 contracted, 45 classified)
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1452 filtered out; finished in 0.00s
```

### Canonical Cairn gates

Pueue task 2130 ran the four exact commands through `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- ... --root . --policy cairn-policy/generated/cairn-policy.json`. Its status and preserved combined transcript were inspected directly. All stages used policy hash `d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c`:

- Validate returned `valid: true`, no issues, 32 specs validated, and 6 active changes. This command does not emit a receipt hash.
- Proposal returned `PASS`, input hash `51b5a3acfe57d796e3a53e16f22da873a2fa267907b0ba81a306d72760b49c49`, receipt hash `06e7e41d74820081b864b35a5b790aa9f85d6403f73ee76b9de48fd5495d9483`.
- Design returned `PASS`, input hash `528ebbcf09ce868683a5dfe06a70f7215ce6efd06fec9ce5b6969b90e8589f30`, receipt hash `4b1ac3001ccdaea5d27b411125d4354c4f860e10b743d5cb2037f7cb98fd5c73`.
- Tasks returned `PASS`, input hash `907c464bab71695aeda2bc8c666cde5b5346297279aed6ab3c8a0f2793bda924`, receipt hash `7f4918fe54b411dd9d6d7eb8818d504e535b93205b9208098c3e5ab6ffff16df`.

### Main integration validation

The validated agent checkpoint was integrated onto current `main` as commit `4713ed6d`, preserving the already-landed production delta/full-fallback path, Wasm component CLI, and KernelScript adapter. Conflict resolution retained both `--remote-delta` and `--remote-observability-config` and combined the production process suites rather than choosing either side.

Pueue task 2200 used a fresh isolated target and passed the integrated binary check, 118 remote-build tests, the 8/7/16/3 immutable-log/exporter/config/trace suites, all seven combined production process tests, 555 `crunch-build` tests, and all six KernelScript integration tests. Its final unconfigured Wasm CLI invocation stopped before test execution because `MANTLE_WASM_COMPONENT_TOOLCHAIN` was absent; that invocation is not counted as passing evidence.

Pueue task 2220 then classified the already-landed `src/wasm_component_cmd.rs` JSON producer in the machine-contract inventory and passed generation, checker self-test, strict freshness validation (`16 contracted, 45 classified`), and all five producer-parity tests. Pueue task 2231 supplied the pinned Nix toolchain at `/nix/store/dyp444vl42jc33wh2sl9qbhc39051gy8-mantle-wasm-component-toolchain-v1` and passed both positive/negative Wasm production CLI tests. These checks prove the observability integration did not erase the current transfer, KernelScript, Wasm, or machine-contract boundaries.

Pueue task 2236 ran the exact-policy Cairn closeout again on integrated `main`: validation covered 38 specs and 10 visible changes with no issues; proposal passed with receipt `4a98bec7a1bdc1d50c097c6f5291b2bba7fb1a08a944619be63665aa82d2a924`; design passed with receipt `b27c93910e023f9e2a8ee2cdd310132621c4f04b698082acdc3df02b9025d87b`; and tasks passed with receipt `96e07f837a2dffb4ea6cd6e17a59b02a1900e4d5a02929962421465f6fefb967`. Every gate used policy hash `d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c` and returned `valid: true` / `PASS`.

### Accepted-spec synchronization

Pueue task 2251 produced an unblocked two-spec dry-run plan with receipt `cdf8f447f43ceb9b19b77edfb3d21f9a8fcc62238cad430a67a4f41c483e8de3`. Pueue task 2253 executed the plan and returned receipt `af3861a3d6c0d3ca932b6c51c94a0843cb6111b497b53019e4cd8fef33ed765d`, but the accepted files remained byte-unchanged despite the mutation receipt. The five delta requirement blocks were therefore materialized explicitly in `cairn/specs/operator-diagnostics/spec.md` and `cairn/specs/remote-builds/spec.md`.

Pueue task 2260 compiled and ran an independent exact-block comparator. It proved each delta requirement occurs exactly once and byte-identically in the accepted spec: operator telemetry (1831 bytes), exporter isolation (1691 bytes), immutable attempt logs (1740 bytes), pure log cursor kernel (1301 bytes), and diagnostic trace context (1119 bytes). Pueue task 2269 then reran exact-policy validation plus proposal/design/tasks gates on the synchronized tree: 38 specs and 10 visible changes validated with no issues, and all three gates returned the same current `PASS` receipts recorded above.

## Scheduler-priority audit

Task 9 is complete. The remote production shell no longer infers priority from queue admission or worker assignment. It creates one immutable bounded ready-root snapshot before dispatch, computes stable BLAKE3 root identities, calls the existing ranking kernel exactly once, preserves the returned order for dispatch, emits one canonical candidate-count event at that real selection boundary, and carries the kernel-produced `PriorityDecisionEvidence` into the strict build JSON report.

The evidence is intentionally narrow. The seam reports ordinary/default preference, structural-fallback history, and zero `KnownGraphPressure`; it does not fabricate critical-path work, blocked roots, or history. The event and metric descriptors expose only declared low-cardinality enums plus the bounded candidate count. Raw roots, labels, store paths, and provider identifiers remain absent from metric labels.

## Decision

The current tree proves immutable attempt logs, bounded production lifecycle telemetry including the real one-shot scheduler-priority decision, transfer accounting, trace propagation/non-authority, exporter isolation, status/build health projection, closed machine-contract parity, and positive/negative process behavior. Publication remains conditional and correctly absent because no production publisher transition occurs.

Telemetry and immutable logs remain diagnostic evidence only. They do not change identity, authorization, fencing, scheduling eligibility, cache identity, transfer identity, output admission, or build truth, and they do not independently prove compiler correctness, reproducibility, release eligibility, CI success, or physical-target determinism.

## Owner

The archived Cairn change `cairn/archive/2026-07-12-persist-remote-attempt-observability/` owns this completed integration and its closeout evidence.

## Archive evidence

Pueue task 2284 produced an unblocked dry-run archive receipt `719ce8883f865cba5b8dcfc995ecd40b38626d19f6a70956f5282a88b9aa06b2`. Pueue task 2285 executed the archive under `CAIRN_ARCHIVE_DATE=2026-07-12` with receipt `1e1eb4d3e8e03c9f8b2fd7a4cd54f5a30609bd6b3adf66da1bf955cace881024` and moved the complete package to the dated archive path.

Pueue task 2297 ran the exact post-archive validation command after the archive and parity evidence were complete:

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 9,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 36,
  "valid": true
}
```

Pueue task 2296 repeated the exact-block comparator against the dated archive paths and reconfirmed the same five byte-identical accepted requirement blocks and byte lengths.

## Next action

No implementation action remains for this archived change. Future consumers must preserve the diagnostic-only non-claims and use separate authority for compiler correctness, reproducibility, release eligibility, CI success, or physical-target determinism.
