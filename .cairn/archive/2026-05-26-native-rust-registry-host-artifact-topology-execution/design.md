# Design

## Execution gate

Topology paths that execute host artifacts for registry-backed packages MUST require two ready fact sets before invoking `rustc` or any build-script executable:

1. `native_registry_source_planning.ready=true` for every registry-backed package participating in the host-artifact topology.
2. `native_host_unit_graph_planning.ready=true` with host-unit facts matching the executable graph.

If either fact set is not ready, Mantle emits a deterministic pre-execution blocker and records zero successful affected executions.

## Registry host source binding

For each registry-backed host unit, receipts preserve the existing source-planning evidence:

- package ID and registry source URL
- lockfile identity and checksum
- declared vendor/source root
- source manifest path
- source digest
- native-vs-Cargo oracle comparison status
- bounded non-claims

Execution receipts for the host unit bind the same source digest, rustc args digest, toolchain identity, produced host artifact digest, and stable receipt hash.

## Host artifact and metadata flow

Supported `proc-macro` host units are compiled first and rebound into target consumer `--extern`/dependency surfaces by artifact digest.

Supported `custom-build` host units are compiled, run with deterministic `OUT_DIR`, and parsed for the already accepted bounded metadata surfaces: `rustc-cfg`, `rustc-env`, `rustc-link-lib`, `rustc-link-search`, and rerun evidence. Target consumers receive the generated metadata through explicit env/arg/link surfaces before their `rustc` execution.

Registry host units use the same topology ordering and receipt identity rules as local/path host units. The new requirement is that registry source facts must also be ready and must cover the registry-backed host package.

## Blockers

Mantle fails closed before host or target execution when:

- registry source planning is not ready
- the host package lacks a matching registry source fact
- checksum, vendor root, manifest, source digest, or oracle evidence is missing/mismatched
- material would come from `$CARGO_HOME`, registry cache, network, git checkout, or target-dir fallback
- native host-unit graph facts are not ready or mismatch the executable graph
- supported build-script metadata is missing, malformed, unsupported, or stale
- produced host artifacts are absent, unreadable, ambiguous, or stale before target execution

## CLI evidence

`rust-plan --execute-topology` and host-artifact topology JSON receipts must retain both plan context and topology execution context so tests can assert:

- `rust_plan.native_registry_source_planning.ready=true` for positive fixtures
- registry host units execute before target consumers
- produced host artifacts/build-script metadata are digested and consumed by target receipts
- negative fixtures block before affected `rustc`/build-script execution
- bounded non-claims exclude Cargo orchestration, network, ambient cache, and general registry compatibility

## Verification

Add focused CLI tests:

- Positive: a local root depends on a vendored registry package with a supported `build.rs` or `proc-macro`; topology execution succeeds without Cargo orchestration after receipt capture.
- Negative: the same shape with missing/stale/unsupported vendor source facts blocks deterministically and does not consult ambient caches.

Run focused Rust checks plus Cairn validation/gates before sync/archive/commit/push.
