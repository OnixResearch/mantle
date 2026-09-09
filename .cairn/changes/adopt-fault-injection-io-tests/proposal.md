## Why

Mantle owns the Nickel build system on the Nix store protocol: derivation evaluation, bounded sandbox builds, cache facts, and build or release evidence. Its failure paths — failed store path reads, failed NAR writes, failed fsync during build commit, failed cache writes — define the bounded build-fact contract. Today those paths are covered only where the store happens to fail.

The komora-io audit (2026-09-09) selected `fault-injection` (Apache-2.0, crates.io) as the stack-wide dev-dependency for deterministic `io::Error` injection with crate, file, and line annotation. The durable-file-publication pilot `adopt-fault-injection-io-tests` proves the fixture pattern first. This change reuses that pattern for the build shell.

## What Changes

- Add `fault-injection` as a pinned dev-dependency of the shell test target.
- Wrap the shell's I/O call sites — store path reads, NAR serialization writes, build-commit fsync, and cache writes — with `fallible!` in test builds.
- Add negative fixtures for each wrapped site asserting the declared bounded build-fact classification.
- Add one positive fixture proving an unchanged build and cache cycle with the default counter.
- Record the dependency as transport `crates.io`, plane `validation`.

## Impact

Immediate outcome: every declared build I/O failure path becomes triggerable from a test. Durable capability contribution: bounded build facts stay provable when the store misbehaves. Maintenance owner: this repository. Repeatability evidence: fixtures set explicit counter values and assert classified outcomes without a damaged store.

## Dependencies

The durable-file-publication pilot `adopt-fault-injection-io-tests` must land first. This change copies its fixture pattern, determinism guard, and dependency-catalog entry shape.

## Non-goals

- Do not add `fault-injection` to any non-dev dependency edge.
- Do not enable `SLEEPINESS` random delays.
- Do not change evaluation, build, cache, or evidence semantics.
- Do not claim sandbox hermeticity, store correctness, or release eligibility from injection coverage.
