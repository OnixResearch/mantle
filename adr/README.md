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
| [0035](0035-cache-rust-units-through-castore-action-results.md) | Cache Rust units through castore action results | Accepted |
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
| [0054](0054-select-snix-backports-by-mantle-compatibility-boundary.md) | Select Snix backports by Mantle compatibility boundary | Accepted |
| [0055](0055-plan-http-cache-closures-before-root-admission.md) | Plan HTTP cache closures before root admission | Accepted |
| [0056](0056-generate-mantlepkgs-from-concrete-package-graphs.md) | Generate Mantlepkgs from concrete package graphs | Proposed |
| [0057](0057-keep-composition-plans-concrete-and-frontend-neutral.md) | Keep composition plans concrete and frontend-neutral | Accepted |
| [0058](0058-limit-store-access-with-concrete-capability-views.md) | Limit store access with concrete capability views | Proposed |
| [0059](0059-bind-source-observations-without-new-signature-authority.md) | Bind source observations without new signature authority | Proposed |
| [0060](0060-verify-remote-admission-with-a-trellis-model.md) | Verify remote admission with a Trellis model | Proposed |
| [0061](0061-keep-remote-bearers-out-of-durable-state.md) | Keep remote bearers out of durable state | Accepted |
| [0062](0062-adapt-ekala-package-maintenance-patterns-without-transferring-authority.md) | Adapt Ekala package-maintenance patterns without transferring authority | Proposed |
| [0063](0063-keep-chaptered-release-archives-as-receipt-bound-transport.md) | Keep chaptered release archives as receipt-bound transport | Accepted |
| [0064](0064-explain-store-retention-before-garbage-collection.md) | Explain store retention before garbage collection | Accepted |
| [0065](0065-reject-picolibc-for-the-stagex-c-runtime.md) | Reject Picolibc for the StageX C runtime | Accepted |
| [0066](0066-select-the-stagex-provider-compiler-by-bounded-workload.md) | Select the StageX provider compiler by bounded workload | Accepted |
| [0067](0067-bind-source-fixed-point-open-file-limits-in-the-proof-plan.md) | Bind source fixed-point open-file limits in the proof plan | Accepted |
| [0068](0068-default-to-deterministic-archives-in-gcc-built-binutils.md) | Default to deterministic archives in GCC-built binutils | Accepted |
| [0069](0069-derive-gcc40-random-seeds-from-main-input-identity.md) | Derive GCC 4.0 random seeds from main input identity | Accepted |
| [0070](0070-bind-source-fixed-point-to-root-action-trust-report.md) | Bind the source fixed point to a root action trust report | Proposed |
| [0071](0071-adopt-nix-archive-at-the-filesystem-nar-boundary.md) | Adopt nix-archive at the filesystem NAR boundary | Proposed |
| [0072](0072-bind-immutable-release-objects-to-one-current-pointer.md) | Bind immutable release objects to one current pointer | Accepted |
| [0073](0073-read-nario-v2-without-transferring-nix-authority.md) | Read Nario v2 without transferring Nix authority | Accepted |
| [0074](0074-enforce-evaluation-budgets-with-an-owned-worker.md) | Enforce evaluation budgets with an owned worker | Accepted |
| [0075](0075-stream-complete-root-outcomes-with-canonical-summaries.md) | Stream complete root outcomes with canonical summaries | Accepted |
| [0076](0076-resolve-historical-nixpkgs-versions-before-mantlepkgs-production.md) | Resolve historical Nixpkgs versions before Mantlepkgs production | Accepted |
| [0077](0077-adopt-nix-derivation-at-the-nix-compatibility-boundary.md) | Adopt `nix-derivation` at the Nix compatibility boundary | Accepted |
| [0078](0078-explore-distributed-evaluation.md) | Explore distributed evaluation feasibility | Accepted |
| [0079](0079-separate-source-fixed-point-evidence-from-working-scratch.md) | Separate source fixed-point evidence from working scratch | Accepted |
| [0080](0080-refresh-source-and-vendor-inputs-as-one-authority-pair.md) | Refresh source and vendor inputs as one authority pair | Accepted |
| [0081](0081-expose-remapped-rust-manifests-through-the-compiler-working-directory.md) | Expose remapped Rust manifests through the compiler working directory | Accepted |
| [0082](0082-compose-promoted-source-proofs-from-stage-checkpoints.md) | Compose promoted source proofs from stage checkpoints | Accepted |
| [0083](0083-bind-receipt-source-through-an-aggregate-closure-root.md) | Bind receipt source through an aggregate closure root | Accepted |
| [0084](0084-isolate-native-prefix-imports-before-downstream-execution.md) | Isolate native-prefix imports before downstream execution | Accepted |
| [0085](0085-bind-seccomp-audit-decisions-to-kernel-responses.md) | Bind seccomp audit decisions to kernel responses | Accepted |
| [0086](0086-supervise-exec-with-ptrace-stops.md) | Supervise exec with ptrace stops instead of user-notify continue | Accepted |
| [0087](0087-acknowledge-ptrace-seize-before-root-stop.md) | Acknowledge ptrace seize before the root stop | Accepted |
| [0088](0088-derive-rust-provider-aggregate-event-bounds.md) | Derive Rust-provider aggregate event bounds from stage bounds | Accepted |
| [0089](0089-resolve-native-bindings-through-the-validated-closure.md) | Resolve native bindings through the validated closure | Accepted |
| [0090](0090-bind-rust-sysroot-relocation-to-the-binding-rewrite.md) | Bind Rust sysroot relocation to the binding rewrite | Accepted |
| [0091](0091-launch-restored-rustc-through-the-bound-loader.md) | Launch restored rustc through the bound loader | Accepted |
| [0092](0092-frame-rust-source-identities-with-blake3.md) | Frame Rust source identities with BLAKE3 | Accepted |
| [0093](0093-resolve-rust-dependency-producers-from-consumed-host-artifacts.md) | Resolve Rust dependency producers from consumed host artifacts | Accepted |
| [0094](0094-resolve-target-dependency-producers-from-the-ready-rust-graph.md) | Resolve target dependency producers from the ready Rust graph | Accepted |
| [0095](0095-bind-rustc-linker-before-ptrace.md) | Bind the rustc linker before ptrace | Accepted |
| [0096](0096-bind-unavailable-rust-tool-probes.md) | Bind unavailable Rust tool probes | Accepted |
| [0097](0097-compose-rust-guard-path-once.md) | Compose the Rust guard path once | Accepted |
| [0098](0098-bind-gcc-subprogram-prefix.md) | Bind the GCC subprogram prefix | Accepted |
| [0099](0099-bind-rust-actions-to-executed-topology.md) | Bind Rust actions to the executed topology | Accepted |
| [0100](0100-normalize-ptrace-request-types.md) | Normalize ptrace request types at the libc boundary | Accepted |
| [0101](0101-export-full-bootstrap-parity-as-an-independent-bundle.md) | Export full-bootstrap parity as an independent bundle | Accepted |
| [0102](0102-vendor-wasi-virt-with-crane.md) | Vendor wasi-virt with Crane | Accepted |
| [0103](0103-retire-bootstrap-markers-through-proof-bound-identities.md) | Retire bootstrap markers through proof-bound identities | Accepted |
| [0104](0104-vendor-spacewasm-with-crane.md) | Vendor SpaceWasm with Crane | Accepted |
| [0105](0105-repair-store-structure-without-changing-authority.md) | Repair store structure without changing authority | Accepted |
