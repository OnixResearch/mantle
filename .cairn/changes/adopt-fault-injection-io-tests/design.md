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

## Placement

Dev-dependency of the shell test target only. The dependency catalog records transport `crates.io`, plane `validation`.

## Verification

Positive coverage: the default counter leaves the store/NAR/cache fixture unchanged; the existing smoke scenario separately proves an actual build and cache cycle.

Negative coverage: injected store-read and NAR-write failures retain their error classes; post-build cache-record write and pre-link fsync failures retain their publication diagnostics at the adapter boundary. Orchestration preserves the ordinary diagnostic and CA-required fail-closed rules separately.

Boundary coverage: annotation carries the expected crate, file, and line; a fixture lock serializes ownership of the shared counter; no fixture enables `SLEEPINESS`.
