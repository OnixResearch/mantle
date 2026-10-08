## Why

Mantle owns the Nickel build system on the Nix store protocol: derivation evaluation, bounded sandbox builds, cache facts, and build or release evidence. Failed store blob reads, failed NAR stream writes, and failed post-build action-result cache-record writes or pre-link fsync have distinct classifications that constrain the bounded build-fact contract. The physical output exporter has no direct commit fsync to inject.

The komora-io audit (2026-09-09) selected `fault-injection` (Apache-2.0, crates.io) as the stack-wide dev-dependency for deterministic `io::Error` injection with crate, file, and line annotation. The durable-file-publication pilot `adopt-fault-injection-io-tests` proves the fixture pattern first. This change reuses that pattern for the build shell.

## What Changes

- Add `fault-injection` as a pinned dev-dependency of the shell test target.
- Wrap the shell's existing I/O call sites — store blob reads, NAR serialization writes, post-build action-result record pre-link fsync, and cache writes — with `fallible!` in test builds.
- Add negative fixtures for each wrapped site asserting its existing store/export classification or cache-publication diagnostic; preserve the distinct ordinary-build and CA-required publication rules.
- Add one positive store/cache fixture with the default counter and prove an unchanged actual build/cache cycle with the existing smoke scenario.
- Record the dependency as transport `crates.io`, plane `validation`.

## Impact

Immediate outcome: the four named store I/O failure sites become triggerable from a test. Durable capability contribution: bounded build facts retain their existing classifications when the store misbehaves; ordinary completed builds record post-build action-result publication failure as a diagnostic, while floating CA and CA-resolved IA intermediates require signed publication and fail as `ca-realisation-untrusted` before dependent consumption. Maintenance owner: this repository. Repeatability evidence: fixtures set explicit counter values and assert classified outcomes without a damaged store.

## Dependencies

The durable-file-publication pilot `adopt-fault-injection-io-tests` must land first. This change copies its fixture pattern, determinism guard, and dependency-catalog entry shape.

## Non-goals

- Do not add `fault-injection` to any non-dev dependency edge.
- Do not enable `SLEEPINESS` random delays.
- Do not change evaluation, build, cache, or evidence semantics.
- Do not claim sandbox hermeticity, store correctness, or release eligibility from injection coverage.
