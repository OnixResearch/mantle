## Goals

- Make every declared build I/O failure path of the shell triggerable from a test.
- Keep injection deterministic, annotated, and confined to test builds.
- Change no evaluation, build, cache, or evidence semantics.

## Mechanism

The `fallible!` macro decrements the process-global `FAULT_INJECT_COUNTER` atomic at each wrapped call site and returns an annotated `io::Error` when it reaches zero. The counter defaults to `u64::MAX`.

Wrapped sites cover castore blob reads, NAR stream writes, post-build action-result cache-record writes, and that record's pre-link fsync. These are existing I/O calls; the physical output exporter has no direct fsync to wrap. Ordinary completed builds record publication faults as diagnostics; floating CA and CA-resolved IA intermediates require signed publication and fail as `ca-realisation-untrusted` before dependent consumption. The pure cores keep planning and classifying from supplied facts and are not touched.

## Determinism controls

- Each fixture stores an explicit counter value and restores the default in a guard.
- Fixtures sharing the process-global counter run serially.
- `SLEEPINESS` stays zero in fixtures and CI.
- The trigger function records the injection site for assertions.
- The armed guard is thread-bound (`!Send`); current-thread fixtures cannot migrate ownership to another runtime worker. Unarmed threads execute the original I/O without consuming the process counter. The async fixture mutex serializes owners, and restoration assertions hold that same mutex to avoid racing the next fixture.
- Boundary fixtures exercise an actual store read on an unarmed OS thread, a queued second owner, and unwind restoration followed by successful cache publication. They do not require a serial test harness to prevent unrelated tests stealing the fault.

## Placement

Dev-dependency of the shell test target only. The dependency catalog records transport `crates.io`, plane `validation`.

## Verification

Positive coverage: the default counter leaves the store/NAR/cache fixture unchanged; the existing smoke scenario separately proves an actual build and cache cycle.

Negative coverage: injected store-read and NAR-write failures retain their error classes; post-build cache-record write and pre-link fsync failures retain their publication diagnostics at the adapter boundary. Orchestration preserves the ordinary diagnostic and CA-required fail-closed rules separately.

Boundary coverage: annotation carries the expected crate, file, and line; a fixture lock serializes ownership of the shared counter; no fixture enables `SLEEPINESS`.

The cache matrix reaches both the record and index through counters 1–4. Record write/fsync faults leave no final record; index write/fsync faults may leave an undiscoverable durable record. All four cases assert temporary-file cleanup, absence of an index hit, and successful retry with the default counter (including duplicate-safe reuse of the orphan record).

The positive cycle compares the complete rendered NAR size and SHA-256 against `crunch-nar`'s filesystem observation, not just archive magic or a payload substring. No injection dependency is needed in `crunch-nar`: the named NAR stream boundary remains the store adapter's copy to its caller-provided writer.

The build orchestration fixture uses a real filesystem publication obstruction after executor completion. It distinguishes ordinary IA diagnostic-only completion from floating CA and CA-resolved IA intermediate failure, and asserts the dependent is not executed. This complements (rather than claims to replace) the store adapter's exact injected write/fsync diagnostics. No production publication rules or cache trust policies change.

These are implementation descriptions, not execution receipts. Current-source fixture execution, the actual build/cache smoke, package/workspace/Clippy/Cairn/Nix gates, and lifecycle acceptance remain unclaimed until the integration owner captures their results.
