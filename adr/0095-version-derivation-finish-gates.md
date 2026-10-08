# ADR 0095: Version derivation finish gates with explicit adoption

## Status

Proposed (2026-10-01). Acceptance depends on real build-time positive and negative proofs and bootstrap family adoption under `standardize-derivation-finish-gates`.

## Context

Bootstrap recipes currently hand-write version, relocation and leak checks. Most use raw `crunch.Derivation`, bypassing `builders/mk_derivation.ncl`. Applying a new default to existing raw records would alter their builds and break the promised staged adoption. Output reference rewriting is a separate change; a finish check must observe, not repair, leaks.

## Decision Drivers

- Failed checks block output admission, and the report records exact bounded observations.
- Existing recipes retain their behavior until explicitly migrated.
- The Nickel contract and the compiled Rust consumer share a versioned deterministic policy export.
- Every adopted raw recipe and every newly constructed builder derivation reaches the same shell path after installation.

## Decision

A closed `mantle-derivation-finish-gates-v1` record enables the common post-install check. Newly constructed `mkDerivation` records default to the policy; previously checked-in raw derivations without a `finish_gates` record retain their old behavior. Raw bootstrap recipes migrate family by family, starting with the GCC wrapper that previously retained build-tree linker paths. Absence in a raw legacy derivation is never described as a passing finish gate. The policy defaults version checking and reference reporting on; relocation and loader auditing are opt-in. Each gate has an explicit `enabled = false` opt-out with a recorded reason at adoption review. Cross-platform references to build-only paths are always denied, even if native leaks are configured as reports. Reference scanning reads existing scanner observations and never rewrites output bytes. Missing declared binaries fail closed. The report block has its own version identifier without changing `crunch-build-report-v1`.

The declared builder script is passed as an argument to a fixed wrapper and executed by a checked child shell: its `exit`, `exec`, or shell syntax cannot skip post-install gates, and recipe-authored log text cannot substitute for a gate's successful exit status.

## Consequences

The compatibility boundary is visible but staged: only adopted raw recipes receive gate enforcement. The default version command for builder records derives from `pname` and the declared `version`; static or non-runnable outputs must opt out explicitly. Empty-environment execution rejects accidental ambient dependencies. Relocation copies may be expensive and stay opt-in. Dlopen auditing requires dynamic Linux loader support and names optional sonames as exceptions, not successful resolutions. These checks make no claim about compiler correctness, global reproducibility, or release eligibility.
