## Why

The immediate consumer is a Mantle developer deciding whether a slow Rust build is blocked by dependency artifacts, a ready queue, or compiler work. Today the ordinary Cargo lane exposes one sandboxed Cargo build, the native host adapter executes whole units serially, and the causal trace explains transitions rather than compiler-phase durations. Neither crate-name guesses nor elapsed build totals justify implementing early-interface execution.

This change proposes a durable, opt-in Rust invocation timeline and repeatable comparison report. Its outcome is an honest explanation of **observed** waits, including unavailable observations and failed runs, before a developer decides whether a readiness optimization is worth its correctness and memory cost. The Rust planning/adapter maintainers own the identity and event mappings; operator-diagnostics maintainers own bounded diagnostic publication. The checked-in small-fixture recipe and retained cohort manifest make that explanation repeatable without a patched compiler or a permanent external benchmark dependency.

The conceptual reference is [Headstart at ff81bfecda2116d54776a7fec538fc2f4e349d1f](https://github.com/PowderworksCode/headstart/tree/ff81bfecda2116d54776a7fec538fc2f4e349d1f), reviewed 2026-10-15: event-based compilation timelines, explicit wait decomposition, and controlled off/on measurement. Its historical performance is not Mantle evidence. Public GitHub visibility is not a copying license; implementation borrows concepts, not upstream scripts, pending license review.

A second conceptual reference is [DioxusLabs/rustd at 131de832c0035b3214289eb65e6a945342180bd8](https://github.com/DioxusLabs/rustd/tree/131de832c0035b3214289eb65e6a945342180bd8): [harness/bench.py](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/harness/bench.py), [harness/BENCH.md](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/harness/BENCH.md), [harness/invalidation.py](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/harness/invalidation.py), and [the multi-crate invalidation report](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/harness/INVALIDATION-regex_syntax+regex_automata+bench2.md). Borrow edit-loop, work-coverage and cold/resident-cost questions, not its scripts, log parsing or benchmark conclusions. These are historical upstream reports, not Mantle measurements or performance promises.

## What Changes

- Define exact invocation, package/unit, resolved-process, and attempt identities, retaining host/target, feature, profile, mode, dependency, and toolchain distinctions. The default Cargo path must get an explicit per-invocation wrapper/event association; no inference from crate names or human progress messages.
- Observe local Rust adapter/session and per-unit sandbox boundaries, compiler start/end, cache disposition, and only metadata/artifact notifications actually supported and emitted by the selected compiler. Treat early-interface, wait, and resume observations as unavailable unless an explicit supported producer supplies them.
- Distinguish dependency eligibility waits, actual ready-queue delay, and observed artifact-ready-to-resume delay. Missing endpoints never become zero-duration phases or synthetic compiler milestones.
- Add a separate opt-in `mantle-rust-wait-diagnostics-v1` sidecar rather than extending or repurposing `mantle-build-trace-v1`. Reuse existing emission/redaction patterns while keeping timings out of receipts and scheduling authority.
- Specify controlled alternating comparisons with equal pinned toolchains, flags, inputs, and resource limits; separate clean, warm/cache, and incremental cohorts; retain all failures/timeouts; summarize successful comparable cohorts with medians and min/max ranges.
- Plan real, offline, bounded chain and wide fixtures exercised through ordinary supported native execution and the default Cargo lane. Distinguish existing full-metadata pipelining from any future early-interface optimization. Include CPU, process-tree or cgroup memory, supported paused-resident counts, time-to-first-error, and drain observations.
- Associate cold/bootstrap, first and subsequent edits, no-op, cache hit/miss, invalidation and error-to-recovery observations with exact content, step and session identities; distinguish disk-result caches, resident query reuse and ordinary Cargo incremental state. Preserve setup, restoration, warm-up and failed outcomes rather than subtracting them to manufacture speedup.
- Report supported work populations and phase boundaries, raw/first-edit samples and per-scenario distributions, plus resident/peak/idle memory and session-lifetime coverage. Require independently qualified equality and work coverage before comparative performance; same-provider cold/warm treatments and separately admitted different-provider reference qualifications are not the same comparison.

## Impact

- **Proposed code surfaces:** pure normalization/reduction adjacent to `mantle-rust-plan-core`; orchestration integration in `mantle-rust-plan-app`; local process/session shell in `src/rust_plan.rs`; explicitly admitted wrapper/event integration for the default `offlineCargoPackage` sandbox; compiler/wrapper and `mantle-cargo-unit-plan` helper observation points; existing diagnostic publication paths. No implementation is included in this package.
- **Compatibility:** disabled remains the existing build behavior and receipt bytes. Enabled diagnostics are bounded and separately versioned; unsupported producers lower coverage, not build correctness. Cache publication remains complete immutable object, signed envelope, then visibility marker.
- **Maintenance:** Rust planning/adapter owners maintain unit mappings and supported-event capability tables on toolchain updates; operator-diagnostics owners maintain redaction, bounds, and output semantics. A future ADR records the diagnostic-only schema and authority boundary before implementation is accepted.
- **Verification planned:** real small-fixture executions, negative incomplete/oversized/sensitive event streams, tracing-on/off receipt and output invariants, and repeated controlled cohorts. These are future tasks; no tests, fixture runs, timings, or implementation proof were produced by this authoring change.

## Non-Goals

- Porting Headstart's rustc/Cargo forks or requiring patched tools for normal diagnostics or baseline measurements.
- Claiming a Mantle speedup, compiler equivalence, complete causal history, or a measured critical path from absent events.
- Replacing Mantle's existing critical-pressure/resource/locality/starvation scheduler or letting diagnostics affect admission, queue selection, cache identity, or cache visibility.
- Extending `mantle-build-trace-v1`, adding performance values to canonical receipt preimages, merging monotonic clocks across hosts/invocations, or making a remote distributed timeline.
- Claiming full-workspace per-unit self-hosting: the documented 256-source-slice lane does not admit Mantle's 711-source workspace.

## Dependencies

No new hard `depends_on` relationship is required. This is independently useful diagnostic groundwork. `validate-rust-pipelining-equivalence` owns semantic differential acceptance; it may reuse the cohort/identity vocabulary but is not replaced by successful timings. `bound-provisional-rust-execution` may supply genuine pause/resume events later. `pipeline-rust-interface-readiness` consumes this package's observed-wait evidence before claiming an optimization; none of those relationships authorizes invented events or introduces a dependency cycle.

Edit-loop reporting remains useful on today's ordinary Cargo/native paths without a resident warm provider, rustd, Headstart or early metadata. A later opt-in bounded warm-workspace experiment may supply supported observations, but is not a prerequisite or new authority. This package does not introduce a scheduler, CAS, shared fragment store or ordinary `PathInfo` promotion; compiler-owned function/query scratch is not a Mantle fragment feature.

The current default Cargo sandbox, normalized native units/process attempts, diagnostic redaction patterns, and existing cache/receipt authority boundaries are prerequisites to preserve, not new authorities. External Headstart rustc base `6006fd03947bd7b04ae5cb5f7f9bb66b44dc8c6e` and Cargo base `4f3fb2428282fb2467c4bb5be0163d2f42a31491` identify the reviewed experiment, not Mantle toolchain changes. Its [unfixed incremental SVH panic](https://github.com/PowderworksCode/headstart/blob/ff81bfecda2116d54776a7fec538fc2f4e349d1f/docs/ice-early-metadata-svh.md) specifically prevents treating clean experimental timings as incremental readiness evidence.

## Success Criteria

1. A developer can associate every reported compiler attempt with its exact selected unit/configuration, or sees an explicit unmapped/unavailable result rather than an inferred crate association.
2. A timeline explains supported compiler intervals and separately labels dependency wait, ready-queue delay, and artifact-ready-to-resume delay with observed endpoints and coverage.
3. Diagnostic-only data remains bounded/redacted and invocation-local; enabling, disabling, losing, or overflowing collection cannot change build/cache/receipt authority.
4. A repeatable real small chain/wide recipe retains every run and reports medians plus ranges only for successful comparable cohorts, including resource and error/drain coverage.
5. Ordinary native and Cargo baselines run without Headstart; full-metadata pipelining and any later early-interface mode are different named comparisons, with no unsupported speedup claim.
6. A developer can inspect content/session-associated edit steps, unavailable or mixed-work metrics, first-edit/raw samples and full bootstrap/resident costs without mistaking an incomplete or unlike-provider report for qualified performance evidence.
