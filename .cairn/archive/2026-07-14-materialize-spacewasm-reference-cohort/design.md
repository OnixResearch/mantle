## Context

SpaceWasm is a small Rust workspace whose primary library is `#![no_std]`, while its host tools, benchmarks, fuzzing helpers, and spectest runner use std-only dependencies. Its support boundary is narrower than the stack's component-model cohort: WebAssembly 1.0 core modules and mutable globals are implemented, while bulk memory, sign extension, non-trapping float-to-int conversion, WIT, the Component Model, and merged WASI support are absent or planned. Its interpreter allocation model also does not itself cap guest linear memory.

The public review that motivated this change found no upstream release, workspace version `0.0.0`, an undeveloped `spacewasm-check` command, manual fuzzing, and open performance work. Those facts make a floating source input or generic “NASA SpaceWasm” label unsuitable as evidence identity.

## Decisions

### 1. One typed profile owns the complete reference cohort

A Nickel profile will identify the exact upstream source commit and archive digest, Cargo lockfile, Rust toolchain, supported host/target matrix, dependency closure, build features, runner configuration, fixture classes, expected support matrix, retention, and non-claims. Runtime Rust will consume a deterministic checked export; it will not evaluate Nickel during builds.

### 2. Source and dependency acquisition are immutable and offline at build time

Mantle will acquire the reviewed source and dependency closure before execution, verify their declared identities, and run the build with network access denied. Branch names, repository default-branch state, mutable GitHub archives, or unpinned Cargo resolution cannot satisfy the profile.

### 3. Library, host runner, and corpus artifacts retain separate identities

The no-std library, std diagnostic runner, support-matrix snapshot, upstream spectests, generated corpus manifests, benchmark inputs, and downstream fixture pack will be distinct bundle members. A runner rebuild or corpus update therefore cannot silently retain an old cohort identity.

### 4. Build validation is bounded and descriptive

The materialization lane will compile the declared targets, run the admitted upstream tests, exercise positive and negative runner fixtures, and record skipped or unavailable checks explicitly. It will not synthesize the missing ground memory checker or treat upstream CI configuration as a passed local check.

### 5. Feature support is data, not inferred prose

The profile will enumerate admitted binary kinds and proposals. Consumer-visible reports will distinguish implemented, unsupported, planned, and unreviewed features. A source update that changes this matrix requires a new cohort identity and complete fixture replay.

### 6. Rehashable bundles bind every parent edge

The bundle will bind source, closure, toolchain, build configuration, binary, fixture, report, and non-claim BLAKE3 identities. Consumers must be able to remeasure each portable member without trusting a Mantle store path or success label.

### 7. Consumer roles remain external

Mantle packages and reports facts. ChaosControl owns differential execution, Molten owns actor admission, Octet owns static artifact policy, Trellis owns local proof claims, Animus owns supported crate surfaces, Kamacite owns adapter/source/value semantics, and Valence/Cairn own their respective evidence/lifecycle interpretations.

### 8. Promotion is explicit and fail-closed

The first bundle is diagnostic/reference-only. A later production consumer must name its own accepted profile and evidence. Missing source, closure, license, support-matrix, negative-fixture, or report members make the bundle incomplete rather than triggering fallback acquisition.

## Functional core / imperative shell split

- **Pure core**: profile validation, cohort identity material, support-matrix comparison, bundle-member planning, parent-edge construction, expected-result evaluation, deterministic diagnostics, and report DTO construction.
- **Imperative shell**: immutable source acquisition, dependency materialization, sandbox setup, Rust compilation, test/fixture execution, artifact hashing, bundle writes, and human/machine rendering.

## Risks / Trade-offs

- Upstream is moving quickly; exact pinning deliberately makes updates noisy and reviewable.
- Host behavior and pointer width can affect interpreter bugs. The profile must identify targets and must not generalize one host result to all embedded targets.
- SpaceWasm contains unsafe allocation/container code. Successful compilation and tests are not a memory-safety audit.
- Imported spectests and fuzz fixtures can have separate licenses or provenance requirements; bundle construction must retain those notices and roles.
- Building a host runner can tempt consumers to treat it as a supported CLI. It remains a bounded diagnostic shell unless a later change defines a product surface.

## Non-Goals

- No fork, vendored maintenance branch, or source patching in this change.
- No implementation of missing WebAssembly proposals, WIT, components, or WASI.
- No flight qualification, formal verification, runtime correctness, sandbox, or release-readiness claim.
- No replacement for Wasmtime or the Mantle component-build cohort.
- No automatic downstream adoption from bundle availability.
