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
| [0025](0025-reserve-remote-resources-with-fenced-leases.md) | Reserve remote resources with fenced leases and verified locality | Accepted |
| [0026](0026-adopt-standalone-nickel-export-core-with-mantle-owned-authority.md) | Adopt the standalone Nickel export core without transferring Mantle authority | Accepted |
| [0027](0027-distinguish-transfer-content-from-chunk-occurrence.md) | Distinguish transfer content identity from chunk occurrence identity | Accepted |
| [0028](0028-bind-registry-oci-metadata-with-a-second-immutable-manifest.md) | Bind registry OCI metadata with a second immutable manifest | Accepted |
| [0029](0029-authenticate-registry-digest-pairs-with-a-signature-artifact.md) | Authenticate registry digest pairs with a signature artifact | Accepted |
| [0030](0030-encode-companion-artifacts-as-oci-image-manifests.md) | Encode companion artifacts as OCI image manifests | Accepted |
| [0031](0031-hydrate-fresh-clone-inputs-from-source-bundles.md) | Hydrate fresh-clone inputs from source bundles | Accepted |
| [0032](0032-deny-live-source-acquisition-in-hydrated-fixed-point-proofs.md) | Deny live source acquisition in hydrated fixed-point proofs | Accepted |
| [0033](0033-preserve-history-when-publishing-mantle.md) | Preserve history when publishing Mantle | Accepted |
| [0034](0034-keep-aeneasverif-proof-and-translation-authority-in-octet.md) | Keep AeneasVerif proof and translation authority in Octet | Accepted |
| [0035](0035-cache-rust-units-through-castore-action-results.md) | Cache Rust units through castore action results | Proposed |
| [0036](0036-run-stagex-grep-bridge-with-a-native-subset-runner.md) | Run the StageX grep bridge with a native subset runner | Accepted |
| [0037](0037-add-a-later-full-shell-for-protected-stagex-configure.md) | Add a later full shell for protected StageX configure | Accepted |
| [0038](0038-stabilize-protected-sed-stdin-with-regular-files.md) | Stabilize protected sed stdin with regular files | Accepted |
| [0039](0039-relocate-stagex-configure-helpers-to-a-bounded-utility.md) | Relocate StageX configure helpers to a bounded utility | Accepted |
| [0040](0040-validate-stagex-generated-sources-before-second-configure.md) | Validate StageX generated sources before second configure | Accepted |
| [0041](0041-run-ylwrap-rewrites-with-a-bounded-native-runner.md) | Run `ylwrap` rewrites with a bounded native runner | Accepted |
| [0042](0042-build-stagex-binutils-archives-with-a-bounded-producer.md) | Build StageX binutils archives with a bounded producer | Accepted |
| [0043](0043-stage-stagex-binutils-install-under-its-logical-prefix.md) | Stage the StageX binutils install under its logical prefix | Accepted |
| [0044](0044-canonicalize-tinycc-local-symbol-names-at-the-stagex-boundary.md) | Canonicalize TinyCC local symbol names at the StageX boundary | Accepted |
| [0045](0045-adopt-orphaned-stagex-exec-descendants-before-inspection.md) | Adopt orphaned StageX exec descendants before inspection | Accepted |
| [0046](0046-realize-foreign-graphs-through-a-receipt-bound-adapter.md) | Realize foreign graphs through a receipt-bound adapter | Proposed |
| [0047](0047-bound-stagex-exec-audits-above-the-closed-binutils-trace.md) | Bound StageX exec audits above the closed binutils trace | Accepted |
| [0048](0048-publish-the-protected-stagex-intermediate-provider.md) | Publish the protected StageX intermediate provider | Accepted |
| [0049](0049-canonicalize-the-stagex-flex-runtime-section-name.md) | Canonicalize the StageX Flex runtime section name | Accepted |
| [0050](0050-build-the-source-fixed-point-through-one-rust-proof-authority.md) | Build the source fixed point through one Rust proof authority | Accepted |
| [0051](0051-cut-legacy-bootstrap-edges-at-the-stagex-provider.md) | Cut legacy bootstrap edges at the StageX provider | Accepted |
| [0052](0052-separate-stagex-execution-evidence-from-runtime-handoff.md) | Separate StageX execution evidence from the runtime handoff | Accepted |
| [0053](0053-isolate-irreversible-seccomp-listeners-by-proof-stage.md) | Isolate irreversible seccomp listeners by proof stage | Accepted |
| [0054](0054-keep-remote-bearers-out-of-durable-state.md) | Keep remote bearers out of durable state | Accepted |
