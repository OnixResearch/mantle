# Design: Publish the Wasm component consumer verifier

## Context

Mantle owns component materialization and the canonical bundle schema. The workspace already has pure `verify_materialization_bundle` logic and a standard-library file verifier. Consumers need a smaller published boundary with stable ownership, explicit limits, and no build or store authority.

## Success Contract

Given parsed bundle data and caller-supplied bounded member observations, the core returns the same ordered verification decision. Given an explicit capability root and bundle file, the shell remeasures every required member and emits one bounded report.

## Decisions

### Keep bundle meaning in a pure core

The core owns schema admission, canonical bundle identity, unique role and path validation, stage-parent linkage, expected member facts, selected runtime-profile binding, ordered blockers, and report payload construction.

The core performs no file, store, process, network, environment, clock, runtime, or output effect. It supports `no_std + alloc` and uses named collection and byte-count limits.

### Publish a narrow consumer contract

The public contract exposes owned DTOs and pure functions. It does not expose Mantle scheduler, derivation, store, builder, cache, release, or CLI types.

Existing `crunch-wasm-component-core` logic can remain the implementation source. The published facade must keep project-facing names and stable schema semantics. Compatibility shims must not duplicate decision logic.

### Resolve files through an explicit capability root

The shell receives one opened root and a relative bundle path. It rejects absolute paths, parent traversal, symlinks, special files, duplicate logical paths, changed file type, and member substitution.

The shell resolves each declared member relative to that root, reads under named bounds, computes BLAKE3, compares declared length and identity, and returns observations to the core. It performs no registry, store, network, or ambient path search.

### Keep structural and file verification separate

Structural verification can run over in-memory bundle data without files. File verification requires a complete caller-owned root. A structural pass must not be labeled as materialized-byte verification.

The final report names which layers ran and records `blocked` when required bytes are unavailable.

### Emit a safe consumer report

The report binds schema, bundle identity, runtime-profile identity, member roles and BLAKE3 values, counts, completed verification layers, blockers, and non-claims. It omits private absolute paths, source payloads, credentials, environment values, and raw tool diagnostics.

### Prove independent consumption

Kamacite supplies one frozen matching and several adversarial consumer fixtures. A second consumer supplies another world and member graph. Mantle tests both through the published facade without importing either product runtime.

Stable publication requires an immutable source revision, API documentation, positive and negative fixture results, and no sibling-path dependency in consumers.

## Functional Core and Imperative Shell

- **Core**: decode-normalized DTO validation, canonical identity, stage linkage, expected observations, decisions, and report payloads.
- **Shell**: capability-root opening, no-follow file reads, byte limits, BLAKE3 measurement, report persistence, and CLI rendering.
- **Consumers**: runtime profile admission, WIT/world meaning, execution, effects, product receipts, and release policy.

## Risks and Controls

- A facade can drift from internal logic. One parity test must compare facade and implementation decisions over the same corpus.
- Locator metadata can become authority. Canonical identity excludes root and display paths.
- A valid bundle can be overclaimed. Reports carry explicit non-claims and reject promoted runtime or correctness roles.

## Non-Claims

A passing verifier report does not prove source trust, compiler correctness, component behavior, runtime isolation, authority, reproducibility, deployment safety, or release eligibility.
