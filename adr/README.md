# Architecture Decision Records

ADRs dated before the 2026-05-14 rename may use the historical Crunch product
name, `crunch` command examples, and legacy project filenames. Current
operator-facing docs use Mantle/mantle names unless they are documenting a
compatibility surface, crate name, or historical decision.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-lazy-goals-vs-eager-dag.md) | Lazy goals vs eager DAG | Proposed |
| [0002](0002-dynamic-derivations.md) | Dynamic derivations | Accepted |
| [0003](0003-configurable-store-prefix.md) | Configurable store prefix | Accepted |
| [0004](0004-native-provenance-attestations.md) | Native attestations | Proposed |
| [0005](0005-project-outputs-schema.md) | Project outputs schema | Proposed |
| [0006](0006-bootstrap-seed-abstraction.md) | Bootstrap seed abstraction | Accepted |
| [0007](0007-normalized-bootstrap-seed-toolchain.md) | Normalized bootstrap seed toolchain | Accepted |
| [0008](0008-reduced-muslcc-seed-provider.md) | Reduced musl.cc seed provider | Accepted |
| [0009](0009-decentralized-release-verification.md) | Decentralized release verification separates technical and social trust | Proposed |
| [0010](0010-keep-mantle-build-tool-boundary.md) | Keep Mantle's boundary build-shaped | Accepted |
| [0011](0011-native-dynamic-plans.md) | Native dynamic plans | Proposed |
| [0012](0012-overlay-store-composition.md) | Overlay store composition | Proposed |
| [0013](0013-remote-execution-hardening.md) | Harden remote execution without replacing Mantle foundations | Proposed |
| [0014](0014-package-ast-grep-as-bounded-structural-evidence.md) | Package ast-grep as bounded structural evidence | Proposed |
| [0015](0015-rust-owned-machine-contract-generation.md) | Keep machine artifacts Rust-owned and generate Nickel review contracts | Accepted |
| [0016](0016-materialize-wasm-components-as-build-artifacts.md) | Materialize WebAssembly components as build artifacts | Proposed |
| [0017](0017-deterministic-lazy-goal-priority.md) | Deterministic priority for lazy ready goals | Accepted |
| [0018](0018-mantle-owned-function-address-release-binding.md) | Own the function-address release binding at the Mantle boundary | Accepted |
| [0019](0019-treat-preserves-function-receipts-as-opaque-canonical-sidecars.md) | Treat Preserves function-address receipts as opaque canonical sidecars | Accepted |
| [0020](0020-keep-trellis-proof-evidence-recorded-only-until-valence-accepts.md) | Keep Trellis proof evidence recorded-only until Valence accepts it | Accepted |
| [0021](0021-publish-release-bundles-with-an-atomic-no-clobber-commit.md) | Publish release bundles with an atomic no-clobber commit | Accepted |
| [0022](0022-bound-kernelscript-as-planning-only-experiment.md) | Bound KernelScript as a planning-only experiment | Proposed |
| [0023](0023-require-content-bound-release-rebuild-authority.md) | Require content-bound authority for release rebuild proofs | Accepted |
| [0024](0024-separate-action-results-from-cas-and-execution.md) | Separate shared action results from CAS and execution | Proposed |
| [0025](0025-reserve-remote-resources-with-fenced-leases.md) | Reserve remote resources with fenced leases and verified locality | Proposed |
