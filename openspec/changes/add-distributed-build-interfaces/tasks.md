# Tasks: Add distributed build interfaces

## Specification

- [x] [serial] S1 Create proposal, design, task checklist, and delta spec for provider-neutral distributed build seams. [covers=distributed-builds.interfaces]

## Implementation / Evidence

- [ ] [serial] I1 Inventory current build scheduler, PathInfo lookup, substitution, store-push, and build-finalization seams; record the concrete modules that will host each interface without adding provider dependencies. [covers=distributed-builds.interfaces.inventory]
- [ ] [depends:I1] I2 Add a pure `RealizationKeyDeriver` core model with deterministic serialization and positive/negative tests for derivation, input closure, platform, toolchain, sandbox/hermeticity, environment, and store-prefix changes. [covers=distributed-builds.interfaces.keys]
- [ ] [depends:I2] I3 Add `ArtifactResolver` and `ArtifactPublisher` traits plus local/in-memory adapters that wrap existing PathInfo/substitution/export behavior without hard-coded remote services. [covers=distributed-builds.interfaces.artifact-adapters]
- [ ] [depends:I3] I4 Refactor the realization pipeline to consult configured artifact resolvers before dispatching realization, preserving existing local-only behavior when no external resolver is configured. [covers=distributed-builds.interfaces.resolver-order]
- [ ] [depends:I1] I5 Add `DerivationRealizer` and `RealizationPolicy` traits with the existing sandbox builder as the required local realizer and fake remote realizers for contract tests. [covers=distributed-builds.interfaces.realizers]
- [ ] [depends:I5] I6 Thread realizer selection through the lazy goal scheduler so independent ready derivations can be placed by policy while goal deduplication, waiter notification, and `max_jobs` bounds remain scheduler-owned. [covers=distributed-builds.interfaces.scheduler-policy]
- [ ] [depends:I5] I7 Define the remote-realization candidate result shape, including logs, output references, worker metadata, and verification status; reject unverified or mismatched remote outputs before PathInfo persistence. [covers=distributed-builds.interfaces.remote-verification]
- [ ] [depends:I3] I8 Add profile/capability configuration structures for resolvers, publishers, and realizers without naming concrete providers in core defaults. [covers=distributed-builds.interfaces.config]
- [ ] [depends:I8] I9 Add operator diagnostics for cache hit, cache miss, publish skipped, local realization, remote realization candidate, remote fallback, and verification rejection events. [covers=distributed-builds.interfaces.diagnostics]

## Verification

- [ ] [depends:I2] V1 Run unit tests for realization-key determinism and negative perturbations. [covers=distributed-builds.interfaces.keys]
- [ ] [depends:I4] V2 Run integration tests proving local-only defaults perform no network/external resolver calls and preserve current realization outcomes. [covers=distributed-builds.interfaces.resolver-order]
- [ ] [depends:I7] V3 Run fake-remote positive and negative contract tests: accepted candidate after verification, rejected digest mismatch, rejected missing log/metadata, and local fallback on remote unavailable. [covers=distributed-builds.interfaces.remote-verification]
- [ ] [depends:I9] V4 Capture operator-facing diagnostics/receipt evidence for cache hit, miss, local realization, remote fallback, and verification rejection. [covers=distributed-builds.interfaces.diagnostics]
- [ ] [serial] V5 Run `openspec validate add-distributed-build-interfaces --strict` and focused Rust/Nix checks for touched crates before marking implementation tasks complete. [covers=distributed-builds.interfaces]
