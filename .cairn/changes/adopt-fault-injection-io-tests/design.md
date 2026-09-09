## Goals

- Make every declared build I/O failure path of the shell triggerable from a test.
- Keep injection deterministic, annotated, and confined to test builds.
- Change no evaluation, build, cache, or evidence semantics.

## Mechanism

The `fallible!` macro decrements the process-global `FAULT_INJECT_COUNTER` atomic at each wrapped call site and returns an annotated `io::Error` when it reaches zero. The counter defaults to `u64::MAX`.

Wrapped sites cover the store boundary: store path reads, NAR serialization writes, build-commit fsync, and cache writes. The pure cores keep planning and classifying from supplied facts and are not touched.

## Determinism controls

- Each fixture stores an explicit counter value and restores the default in a guard.
- Fixtures sharing the process-global counter run serially.
- `SLEEPINESS` stays zero in fixtures and CI.
- The trigger function records the injection site for assertions.

## Placement

Dev-dependency of the shell test target only. The dependency catalog records transport `crates.io`, plane `validation`.

## Verification

Positive coverage: default counter completes a build and cache cycle unchanged.

Negative coverage: injected failures at store read, NAR write, commit fsync, and cache write, each asserting the declared bounded build-fact classification.

Boundary coverage: annotation carries the expected crate, file, and line; parallel fixtures never observe a foreign counter; no fixture enables `SLEEPINESS`.
