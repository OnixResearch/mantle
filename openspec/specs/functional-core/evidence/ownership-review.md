# Ownership Review

Status: adopted-core boundary review synchronized to main spec assets
Date: 2026-08-03

## Legacy std paths reduced to adapter-only form

### `crunch-attestation`

- `crates/crunch-attestation/src/canonical.rs` → adapter-only re-export
- `crates/crunch-attestation/src/digest.rs` → adapter-only re-export
- `crates/crunch-attestation/src/error.rs` → adapter-only re-export
- `crates/crunch-attestation/src/policy.rs` → adapter-only re-export
- `crates/crunch-attestation/src/release.rs` → adapter-only re-export
- `crates/crunch-attestation/src/schema.rs` → adapter-only re-export
- `crates/crunch-attestation/src/version.rs` → adapter-only re-export

### `crunch-project`

- `crates/crunch-project/src/manifest.rs` → adapter-only re-export
- `crates/crunch-project/src/lock.rs` → adapter-only re-export
- `crates/crunch-project/src/version.rs` → adapter-only re-export
- `crates/crunch-project/src/merge.rs` → adapter-only re-export
- `crates/crunch-project/src/generate.rs` → adapter-only re-export
- `crates/crunch-project/src/drift.rs` → adapter-only re-export
- `crates/crunch-project/src/mirrors.rs` → adapter-only re-export
- `crates/crunch-project/src/refresh.rs` → adapter-only re-export
- `crates/crunch-project/src/upgrade.rs` → adapter-only wrapper over core upgrade error/result

## Touched std workspace source files outside the legacy paths

- `crates/crunch-attestation/src/adapter.rs` → `adapter-only`
  - borrowed/std-facing attestation wrappers translate inputs into owned core
    request types before calling `crunch-attestation-core`
- `crates/crunch-attestation/src/discovery.rs` → `unrelated`
  - std shell file discovery stays here intentionally; this is effectful
    directory/file loading, not first-wave business logic moved back out of core
- `crates/crunch-attestation/src/lib.rs` → `adapter-only`
  - crate-root module wiring and re-exports for the std-facing attestation shell
- `crates/crunch-project/src/attestation.rs` → `adapter-only`
  - reduced to a re-export shim over `attestation_adapter.rs`
- `crates/crunch-project/src/attestation_adapter.rs` → `adapter-only`
  - borrowed/std-facing project-attestation wrapper builds owned
    `ProjectAttestationRequest` values for `crunch-project-core`
- `crates/crunch-project/src/error.rs` → `adapter-only`
  - std-facing error translation keeps `serde_json::Error` and `std::io::Error`
    outside the no-std core boundary
- `crates/crunch-project/src/lib.rs` → `adapter-only`
  - crate-root module wiring and `pub use` exports for the std-facing shell;
    no first-wave project business logic lives here
- `crates/crunch-project/src/mirrors.rs` → `unrelated`
  - touched while the first-wave project-core surface changed, but it remains a
    thin mirror helper re-export and does not reintroduce moved business logic
- `crates/crunch-project/src/refresh_adapter.rs` → `adapter-only`
  - resolver/file/network hashing stays in the std adapter while the core sees
    only owned normalized refresh requests and outcomes
- `crates/crunch-project/src/upgrade_adapter.rs` → `adapter-only`
  - keeps CLI-facing upgrade error translation in std while
    `crates/crunch-project/src/upgrade.rs` stays a re-export shim
- `crates/crunch-project/tests/integration_nickel.rs` → `unrelated`
  - integration coverage for generated Nickel/import behavior; no first-wave
    business logic moved back into std production code
- `crates/crunch-shell/src/adapter.rs` → `adapter-only`
  - std-facing shell activation translation keeps `PathBuf`, `split_paths(...)`,
    non-UTF-8 rejection, and `ExecTarget` reconstruction outside
    `crunch-shell-core`
- `crates/crunch-shell/src/lib.rs` → `adapter-only`
  - crate-root re-exports and std shell wiring remain thin adapters around
    `crunch-shell-core`
- `crates/crunch-shell/src/types.rs` → `adapter-only`
  - std-owned path and `OsString` shell-facing types stay here while
    `crunch-shell-core` public APIs stay on owned UTF-8 data
- `crates/crunch-delta/src/lib.rs` → `adapter-only`
  - crate-root re-exports and std-facing compatibility wiring stay thin around
    `crunch-delta-core`
- `crates/crunch-delta/src/fixtures.rs` → `unrelated`
  - benchmark-only fixture construction and wire-size modeling remain std-owned
    helper code; this file does not contain production delta planning/protocol
    business logic
- `crates/crunch-delta/src/model.rs` → `adapter-only`
  - std-facing `B3Digest` / `HashSet` façades and conversion helpers translate
    runtime-shaped values into owned `crunch-delta-core` requests before core
    calls
- `crates/crunch-delta/src/negotiation.rs` → `adapter-only`
  - façade-owned wire types and `std::error::Error` mapping stay in std while
    negotiation semantics remain in `crunch-delta-core`
- `crates/crunch-delta/src/planner.rs` → `adapter-only`
  - std-facing `PlanError` and borrowed façade conversion remain here while the
    transfer-planning logic itself stays in `crunch-delta-core`
- `crates/crunch-delta/src/manifest.rs` → `adapter-only`
  - castore probing and receiver-manifest construction remain std-owned shell
    work before the core boundary
- `crates/crunch-delta/src/substitution.rs` → `adapter-only`
  - async store/network/session orchestration and attestation integration stay
    in the std adaptor around `crunch-delta-core`
- `crates/crunch-store/src/gc.rs` → `adapter-only`
  - service and filesystem observations become normalized owned facts before
    `crunch-gc-core` computes reachability, candidates, and mutation disposition
  - deletion, database rewrites, and report serialization remain in std
- `crates/crunch-store/src/repair.rs` → `adapter-only`
  - NAR rendering and authority checks become normalized facts before
    `crunch-repair-core` decides admission, transaction order, and report status
  - signing, sidecar publication, persistence, verification, and rollback
    execution remain in std
- `src/attest_cmd.rs` → `unrelated`
  - root CLI shell for attestation commands, store access, and output formatting
- `src/project_cmd.rs` → `unrelated`
  - root CLI shell still owns manifest/lock file I/O, user-facing output, and
    command orchestration around the std `crunch-project` API
- `src/release_evidence.rs` → `adapter-only`
  - proof-bundle loading, file copying, directory hashing, manifest I/O, and
    digesting remain in the std shell while canonical manifest/proof validation
    stays in `crunch-release-core`
- `src/release_cmd.rs` → `adapter-only`
  - release CLI path resolution and output formatting stay in the std shell
    around `crunch-release-core`
- `tests/attest_cli.rs` → `unrelated`
  - CLI integration coverage only; no first-wave core logic reintroduced
- `tests/project_cli.rs` → `unrelated`
  - CLI smoke coverage only; no first-wave core logic reintroduced
- `tests/project_refresh_cli.rs` → `unrelated`
  - CLI refresh integration coverage only; no first-wave core logic reintroduced

## Scoped adapter enrollment source review (2026-10-18)

The canonical adopted-core inventory now includes the CMake, ESP-IDF, FPGA,
GPU, model-conversion, pnpm, Python, unified-test, verification-bridge and
Zephyr functional cores. Host/wasm checks and core tests consume that same
inventory; dependency, purity, scope and API-shape checks retain their existing
rules. This is source enrollment, not an executed gate or real-tool acceptance
result. Initial enrollment exposed borrowed public parameters and the test
core's `impl Serialize` boundary, incompatible with the existing owned-data
API-shape policy. Enrollment does not waive those rules: the integrated owned-API
cutover must satisfy the same canonical checks before acceptance is claimed.

- `crates/crunch-cmake/src/lib.rs` → `adapter-only`
  - invokes the validated core plan, binds filesystem/tool members, runs
    configure/build/install, inspects staged files and emits provenance;
    native-input translation uses the existing Cargo native-input boundary
- `crates/crunch-esp-idf/src/lib.rs` → `adapter-only`
  - converts validated plans to ordinary derivations, checks logical store-root
    spelling and launches the embedded driver with the declared Python input
- `crates/crunch-fpga/src/lib.rs` → `adapter-only`
  - measures/snapshots declared files, executes bounded synthesis stages through
    the existing hardware sandbox and publishes inspected stage artifacts
- `crates/crunch-gpu/src/lib.rs` → `adapter-only`
  - bounds admitted filesystem members, invokes declared compiler/inspector
    tools, inspects actual ELF bytes and writes ordinary derivation provenance
- `crates/crunch-model-conversion/src/lib.rs` → `adapter-only`
  - serializes the validated conversion plan, declares input roots and embeds
    the Python runner/independent reader in an ordinary derivation
- `crates/crunch-pnpm/src/lib.rs` → `adapter-only`
  - measures declared trees, invokes pinned pnpm with a private workspace,
    checks observed links/lock bytes and copies declared output files
- `crates/crunch-python/src/main.rs` → `adapter-only`
  - reads bounded configuration, invokes pure admission/native-input helpers,
    checks evaluated input bindings, measures inputs and launches the worker
- `crates/crunch-test/src/lib.rs` → `adapter-only`
  - performs bounded filesystem/process/coverage operations for `TestPort`,
    lowers ordinary recipes and checks admitted result identity/structure
- `crates/crunch-verification-bridge/src/lib.rs` → `adapter-only`
  - reads bounded selected-root material, invokes identity-only consumers and
    exports exact bytes; it neither executes a verifier nor admits a theorem
- `crates/crunch-zephyr/src/lib.rs` → `adapter-only`
  - measures filesystem trees, checks observed tool/source cohorts, invokes
    configure/build and publishes inspected firmware artifacts

`crunch-test-app` is intentionally not classified as a pure functional core:
its `TestPort` trait sequences effects supplied by the std adapter. It remains
in ordinary first-party quality coverage. Neither this distinction nor the
classification above widens the supported adapter cohorts, grants store trust,
or asserts that real build/firmware/model/proof acceptance has been exercised.

Scoped enrollment reviewer: AdapterEnrollment (source inspection only).

## Review verdict

- No first-wave attestation foundational business logic remains in the reduced
  std legacy files listed above.
- No first-wave project foundational business logic remains in the reduced std
  legacy files listed above.
- No first-wave business logic was reintroduced in the touched std workspace
  source files derived from the change history outside the legacy paths.
- Remaining first-wave shell-owned logic still intentionally lives in std files
  called out by `workspace-inventory.md` (`discovery.rs`, `refresh_adapter.rs`,
  CLI shells, and related adapters) until a later extraction wave explicitly
  reshapes those APIs.
- shell/release business logic remains in `crunch-shell-core` and `crunch-release-core`, while delta planning/protocol business logic remains in `crunch-delta-core` rather than drifting back into the reviewed std adapter files.
- GC and final-NAR repair decisions remain in `crunch-gc-core` and
  `crunch-repair-core`. Their std adapters retain observations and effects only.

Reviewer: pi session


## History-derived and prospective source ownership supplement (2026-10-18)

This supplement preserves the first-wave scope and review verdict above. It
repairs path-ledger completeness; it does not change the checker, its history
derivation, the canonical inventory, or any policy. `unrelated` means unrelated
to the adopted migration's extracted ownership, **not** globally free of business
logic and **not** an exemption from another core/adapter, purity, authority or
quality boundary. `adapter-only` describes the listed consumer's role at that
boundary; observed-byte validation, host error translation and executing core
intents remain shell work. It is not a claim of full runtime validation.

### Derivation and source-review basis

The metadata-only derivation used
`scripts/no_std_core_checks.py::touched_std_workspace_source_files` and the
canonical `validation/adopted-core-inventory.toml`, without invoking a checker
command or gate. The resolved history roots are
`openspec/changes/archive/2026-04-22-no-std-functional-core` and
`openspec/changes/archive/2026-04-23-delta-functional-core`. The first history
revision is `6e47f8725126943b26335abd6d1a6df4606fa3e8`; its parent,
`00190e4d1096905bf244d6aadf1664c7fd822625`, is the unchanged diff base. At
reviewed HEAD `43ccd68840b748e03773fc1bff5eb1cfca99dffa`, the predicate derives
647 historical paths; union with the 20 required adapter paths is 657 paths.
The existing exact-syntax ledger covers 36 of that union, leaving 621 missing
historical/required classifications.

Prospective coverage additionally includes eligible tracked worktree changes
and non-ignored untracked Rust sources present during this review, using the
same source predicate and legacy exclusions. There are 186 eligible prospective
paths, 94 outside the historical/required union. The 715 appended rows below
therefore cover the 621 missing historical/required paths plus those 94 new
paths: 36 `adapter-only` and 679 `unrelated`. Together with the 40 existing exact
classifications (including four retained entries outside this union), the
ledger has 755 explicit rows for 751 currently required-or-prospective paths
plus those four retained entries. This is a metadata snapshot, not a waiver for
future source paths; newly added paths still require source ownership review.

Source-family decisions use the original migration's
`openspec/changes/archive/2026-04-22-no-std-functional-core/evidence/workspace-inventory.md`,
the adopted inventory and existing enrollment review, crate-root ownership and
module declarations, and targeted consumer/adapter inspection. The original
inventory assigns attestation schema/canonical/digest/release-policy transforms
and project manifest/lock/merge/drift/refresh/application/synthesis to their
cores; it leaves discovery, resolver/file I/O and CLI orchestration in std.
The supplement does not replace that scope with a global assertion about every
later algorithm in the historical diff. No first-wave moved algorithm was
identified as reimplemented in std by this source-family review. The grouped
caveats below expressly retain other domains' real decision logic and existing
architectural obligations. No gate, build, test, lint, format, smoke, real-tool
or remote workflow was run for this supplement.

### Additional enrolled builder and evidence entry adapters

These are the std command entry points around the enrolled CMake, ESP-IDF,
FPGA, GPU, model-conversion, pnpm, unified-test, verification-bridge and Zephyr
APIs already reviewed above. They decode CLI/environment/file inputs, delegate
plan decisions to their core or std façade, and render or execute through the
existing adapter. The pnpm `admission.rs` and `files.rs` additionally decode and
measure observed lock/archive/filesystem material, check integrity and bound
host operations; pinned pnpm still owns dependency resolution. The verification
bridge entry point exports identity-linked bytes, not proof acceptance.

- `crates/crunch-cmake/src/main.rs` → `adapter-only`
- `crates/crunch-esp-idf/src/main.rs` → `adapter-only`
- `crates/crunch-fpga/src/main.rs` → `adapter-only`
- `crates/crunch-gpu/src/main.rs` → `adapter-only`
- `crates/crunch-model-conversion/src/main.rs` → `adapter-only`
- `crates/crunch-pnpm/src/admission.rs` → `adapter-only`
- `crates/crunch-pnpm/src/files.rs` → `adapter-only`
- `crates/crunch-pnpm/src/main.rs` → `adapter-only`
- `crates/crunch-test/src/main.rs` → `adapter-only`
- `crates/crunch-verification-bridge/src/main.rs` → `adapter-only`
- `crates/crunch-zephyr/src/main.rs` → `adapter-only`

### Adopted release-core consumer adapters

These modules load and measure evidence, translate observed bytes and capability
roots into release-core inputs, invoke core validation/planning/canonicalization,
or execute publication/copy/transport intents and render results. For example,
`ast_grep_evidence.rs::read_ast_grep_evidence` passes measured text to
`parse_ast_grep_structural_evidence_json`; `cairn_release_handoff.rs` calls
`validate_cairn_release_evidence_handoff`; `content_bound_requirement_evidence.rs`
measures files and calls `seal_content_bound_evidence_row`; immutable-pointer,
chapter-transport, tree-copy, rebuild-authority and reproducibility consumers use
the corresponding release-core plans/identities. Host byte checks and error
translation are not a second implementation of the first-wave attestation
canonicalization or project refresh decisions. This is scoped source ownership,
not certification of these workflows or their evidence.

- `src/ast_grep_evidence.rs` → `adapter-only`
- `src/cairn_release_handoff.rs` → `adapter-only`
- `src/content_bound_requirement_evidence.rs` → `adapter-only`
- `src/function_address_binding_cmd.rs` → `adapter-only`
- `src/global_reproducibility_cmd.rs` → `adapter-only`
- `src/global_reproducibility_release.rs` → `adapter-only`
- `src/rebuild_authority.rs` → `adapter-only`
- `src/release_attestation.rs` → `adapter-only`
- `src/release_capability.rs` → `adapter-only`
- `src/release_chapter_transport.rs` → `adapter-only`
- `src/release_current_pointer.rs` → `adapter-only`
- `src/release_nix_witness.rs` → `adapter-only`
- `src/release_publication.rs` → `adapter-only`
- `src/release_reproducibility.rs` → `adapter-only`
- `src/release_source.rs` → `adapter-only`
- `src/release_tree_copy.rs` → `adapter-only`
- `src/verification_gauntlet_cmd.rs` → `adapter-only`

### Adopted project and shell consumer adapters

Bootstrap-pin and pin-import commands collect external pin facts and execute
project-core plans; file generation applies verified core operations.
`project_resolve.rs::LiveResolver` implements the std `RefreshResolver` boundary
with filesystem, VCS, network and hashing observations; it does not replace the
core's normalized refresh planning/application. `project_retention_fact.rs`
performs one bounded file read. `shell_cmd.rs` snapshots host environment,
builds/reads the sidecar, delegates activation through `crunch-shell` and executes
the result. Its root/lease registration is host integration, not a claim that
all retention policy was extracted with activation.

- `src/bootstrap_pin_cmd.rs` → `adapter-only`
- `src/filegen_cmd.rs` → `adapter-only`
- `src/pin_import.rs` → `adapter-only`
- `src/project_resolve.rs` → `adapter-only`
- `src/project_retention_fact.rs` → `adapter-only`
- `src/shell_cmd.rs` → `adapter-only`

### Adopted attestation and retention store translations

`attestation.rs` observes PathInfo, provenance and runtime references, translates
those store-shaped facts into attestation graphs, persists files, and calls the
attestation façade for canonical bytes/digests. Graph assembly here is store-fact
translation, not duplicate schema, canonicalization, digest or policy evaluation.
`retention.rs` decodes the checked-in runtime policy and root records into
`crunch_gc_core::retention` facts, including policy-identity/size checks before
the core boundary. Neither row declares the entire store crate adapter-only.

- `crates/crunch-store/src/attestation.rs` → `adapter-only`
- `crates/crunch-store/src/retention.rs` → `adapter-only`

### Other decision and contract families outside the adopted inventory

These source families own distinct domains: action-result/artifact admission,
Android/ELF admission, bootstrap lineage/provider capabilities, composition and
user profiles, evaluation budgets/streams/watch state, hardware simulation,
KernelScript, Nix gateway admission, overlays, remote execution, Rust cache and
compiler slots, service readiness, source observation/ingest, SpaceWasm,
WebAssembly components, public build/Cargo/portable-client contracts, Rust-plan
and differential evidence, causal traces, lock/vendor planning and Mantlepkgs.
Their root module declarations and exports assign those decisions to those
families, rather than to the first-wave attestation/project module list.

Many declare `#![no_std]`; the history predicate nevertheless includes them
because it excludes only canonical adopted crate directories, not every crate
with a `-core` name. Do not infer no-std status from this grouping:
`crunch-action-result-core/src/lib.rs` and
`crunch-rust-cache-core/src/lib.rs` use `std::collections` in the reviewed source.
They and every other family remain subject to their own applicable boundaries.
This classification neither enrolls them in the adopted inventory nor labels
their real admission/planning/business logic as absent.

- `crates/crunch-action-result-core/src/artifact_auth.rs` → `unrelated`
- `crates/crunch-action-result-core/src/error.rs` → `unrelated`
- `crates/crunch-action-result-core/src/lib.rs` → `unrelated`
- `crates/crunch-android-core/src/elf.rs` → `unrelated`
- `crates/crunch-android-core/src/lib.rs` → `unrelated`
- `crates/crunch-android-core/src/native.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/error.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/lib.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/lineage.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/nominal.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/profile.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/provider_boundary.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/source_root_capability.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/stagex.rs` → `unrelated`
- `crates/crunch-bootstrap-core/src/validate.rs` → `unrelated`
- `crates/crunch-composition-core/src/lib.rs` → `unrelated`
- `crates/crunch-composition-core/src/user_profile.rs` → `unrelated`
- `crates/crunch-eval-budget-core/src/lib.rs` → `unrelated`
- `crates/crunch-evaluation-stream-core/src/identity.rs` → `unrelated`
- `crates/crunch-evaluation-stream-core/src/ledger.rs` → `unrelated`
- `crates/crunch-evaluation-stream-core/src/lib.rs` → `unrelated`
- `crates/crunch-evaluation-stream-core/src/projection.rs` → `unrelated`
- `crates/crunch-evaluation-stream-core/src/types.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/digest.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/error.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/evidence.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/lib.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/model.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/plan.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/profile.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/smoke.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/blocker.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/compiler.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/digest.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/generated.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/handoff.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/inspection.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/lib.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/model.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/planner.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/profile.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/receipt.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/target.rs` → `unrelated`
- `crates/crunch-nix-gateway-core/src/lib.rs` → `unrelated`
- `crates/crunch-overlay-core/src/lib.rs` → `unrelated`
- `crates/crunch-remote-core/src/admission.rs` → `unrelated`
- `crates/crunch-remote-core/src/attempt.rs` → `unrelated`
- `crates/crunch-remote-core/src/attempt_log.rs` → `unrelated`
- `crates/crunch-remote-core/src/caveat.rs` → `unrelated`
- `crates/crunch-remote-core/src/demand.rs` → `unrelated`
- `crates/crunch-remote-core/src/effect.rs` → `unrelated`
- `crates/crunch-remote-core/src/endpoint.rs` → `unrelated`
- `crates/crunch-remote-core/src/external_batch.rs` → `unrelated`
- `crates/crunch-remote-core/src/failure.rs` → `unrelated`
- `crates/crunch-remote-core/src/identity.rs` → `unrelated`
- `crates/crunch-remote-core/src/lease.rs` → `unrelated`
- `crates/crunch-remote-core/src/lib.rs` → `unrelated`
- `crates/crunch-remote-core/src/nominal.rs` → `unrelated`
- `crates/crunch-remote-core/src/output.rs` → `unrelated`
- `crates/crunch-remote-core/src/protocol.rs` → `unrelated`
- `crates/crunch-remote-core/src/provider.rs` → `unrelated`
- `crates/crunch-remote-core/src/receipt.rs` → `unrelated`
- `crates/crunch-remote-core/src/recovery.rs` → `unrelated`
- `crates/crunch-remote-core/src/resource.rs` → `unrelated`
- `crates/crunch-remote-core/src/resource_policy.rs` → `unrelated`
- `crates/crunch-remote-core/src/resource_quota.rs` → `unrelated`
- `crates/crunch-remote-core/src/transfer.rs` → `unrelated`
- `crates/crunch-rust-cache-core/src/cc.rs` → `unrelated`
- `crates/crunch-rust-cache-core/src/error.rs` → `unrelated`
- `crates/crunch-rust-cache-core/src/lib.rs` → `unrelated`
- `crates/crunch-rust-cache-core/src/shared.rs` → `unrelated`
- `crates/crunch-rust-cache-core/src/slots.rs` → `unrelated`
- `crates/crunch-rust-cache-core/src/wrapper.rs` → `unrelated`
- `crates/crunch-service-readiness-core/src/admission.rs` → `unrelated`
- `crates/crunch-service-readiness-core/src/evaluation.rs` → `unrelated`
- `crates/crunch-service-readiness-core/src/lib.rs` → `unrelated`
- `crates/crunch-service-readiness-core/src/model.rs` → `unrelated`
- `crates/crunch-source-core/src/digest.rs` → `unrelated`
- `crates/crunch-source-core/src/ingest.rs` → `unrelated`
- `crates/crunch-source-core/src/lib.rs` → `unrelated`
- `crates/crunch-source-core/src/model.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/admission.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/bundle.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/diagnostic.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/digest.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/lib.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/model.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/profile.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/report.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/results.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/run_record.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/stable_report.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/support.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/admission.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/blocker.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/bundle.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/composition.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/digest.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/evidence.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/generated.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/lib.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/manifest.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/model.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/report.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/virtualization.rs` → `unrelated`
- `crates/crunch-watch-core/src/lib.rs` → `unrelated`
- `crates/mantle-build-contract/src/admission.rs` → `unrelated`
- `crates/mantle-build-contract/src/identity.rs` → `unrelated`
- `crates/mantle-build-contract/src/lib.rs` → `unrelated`
- `crates/mantle-build-contract/src/model.rs` → `unrelated`
- `crates/mantle-cargo-build-bundle-core/src/application.rs` → `unrelated`
- `crates/mantle-cargo-build-bundle-core/src/lib.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/src/build_script.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/src/lib.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/src/producer.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/src/source_nar.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/src/unit_helper.rs` → `unrelated`
- `crates/mantle-causal-trace-core/src/lib.rs` → `unrelated`
- `crates/mantle-lock-vendor-core/src/lib.rs` → `unrelated`
- `crates/mantle-lock-vendor-core/src/shared_table.rs` → `unrelated`
- `crates/mantle-portable-client-core/src/lib.rs` → `unrelated`
- `crates/mantle-rust-differential-core/src/edits.rs` → `unrelated`
- `crates/mantle-rust-differential-core/src/lib.rs` → `unrelated`
- `crates/mantle-rust-differential-core/src/manifest.rs` → `unrelated`
- `crates/mantle-rust-differential-core/src/population.rs` → `unrelated`
- `crates/mantle-rust-differential-core/src/recovery.rs` → `unrelated`
- `crates/mantle-rust-differential-core/src/semantics.rs` → `unrelated`
- `crates/mantle-rust-differential-core/src/waits.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/build_script.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/cfg.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/digest.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/features.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/lib.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/model.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/plan.rs` → `unrelated`
- `crates/mantle-rust-plan-core/src/receipt.rs` → `unrelated`
- `crates/mantlepkgs-core/src/catalog.rs` → `unrelated`
- `crates/mantlepkgs-core/src/domains.rs` → `unrelated`
- `crates/mantlepkgs-core/src/error.rs` → `unrelated`
- `crates/mantlepkgs-core/src/impact.rs` → `unrelated`
- `crates/mantlepkgs-core/src/lib.rs` → `unrelated`
- `crates/mantlepkgs-core/src/manifest.rs` → `unrelated`
- `crates/mantlepkgs-core/src/render.rs` → `unrelated`
- `crates/mantlepkgs-core/src/updates.rs` → `unrelated`
- `crates/mantlepkgs-core/src/versions.rs` → `unrelated`

### Application contracts and effect-sequencing families

`mantle-application-contract` owns command/effect/capability vocabulary and
application plans; the remote and Rust-plan application crates sequence their
respective ports. `crunch-test-app` is likewise an application, not an enrolled
pure core: its `TestPort` trait and `execute` sequence preparation, capture and
coverage effects while delegating result classification/aggregation to
`crunch-test-core`. A no-std declaration alone does not make port orchestration
a pure decision core or authorize a public trait in the adopted core surface.
These rows preserve that distinction without exempting application policy from
other architectural rules.

- `crates/crunch-remote-app/src/lib.rs` → `unrelated`
- `crates/crunch-test-app/src/lib.rs` → `unrelated`
- `crates/mantle-application-contract/src/admission_facts.rs` → `unrelated`
- `crates/mantle-application-contract/src/artifact.rs` → `unrelated`
- `crates/mantle-application-contract/src/attest.rs` → `unrelated`
- `crates/mantle-application-contract/src/bootstrap_flow.rs` → `unrelated`
- `crates/mantle-application-contract/src/component_flow.rs` → `unrelated`
- `crates/mantle-application-contract/src/diagnostics.rs` → `unrelated`
- `crates/mantle-application-contract/src/envelope.rs` → `unrelated`
- `crates/mantle-application-contract/src/evaluation.rs` → `unrelated`
- `crates/mantle-application-contract/src/family.rs` → `unrelated`
- `crates/mantle-application-contract/src/graph_inspection.rs` → `unrelated`
- `crates/mantle-application-contract/src/graph_query.rs` → `unrelated`
- `crates/mantle-application-contract/src/lib.rs` → `unrelated`
- `crates/mantle-application-contract/src/nix_free_demo.rs` → `unrelated`
- `crates/mantle-application-contract/src/output_format.rs` → `unrelated`
- `crates/mantle-application-contract/src/planning.rs` → `unrelated`
- `crates/mantle-application-contract/src/ports.rs` → `unrelated`
- `crates/mantle-application-contract/src/project_lifecycle.rs` → `unrelated`
- `crates/mantle-application-contract/src/realization.rs` → `unrelated`
- `crates/mantle-application-contract/src/release.rs` → `unrelated`
- `crates/mantle-application-contract/src/remote_execution.rs` → `unrelated`
- `crates/mantle-application-contract/src/root_validators.rs` → `unrelated`
- `crates/mantle-application-contract/src/source_provenance.rs` → `unrelated`
- `crates/mantle-application-contract/src/store_administration.rs` → `unrelated`
- `crates/mantle-application-contract/src/store_prefix.rs` → `unrelated`
- `crates/mantle-rust-plan-app/src/application.rs` → `unrelated`
- `crates/mantle-rust-plan-app/src/lib.rs` → `unrelated`
- `crates/mantle-rust-plan-app/src/ports.rs` → `unrelated`

### Build, evaluator, conversion, pipeline and store runtime families

The original migration inventory explicitly left the evaluator, glue, build,
store and pipeline tiers outside its first-wave pure module extraction. These
paths cover Nickel/session/import handling, derivation conversion, build and
dynamic-plan orchestration, sandbox/fetch/process execution, remote service
integration, persistence/capabilities, export/archives, roots and cache transport;
`crunch-nar` handles filesystem NAR rendering/interoperability. Dedicated adopted
GC/repair and attestation/retention translations are classified separately.

Some modules contain deterministic admission, scheduling, identity and policy
helpers alongside effects; `unrelated` records distinct ownership, not a blanket
claim that these std families have no business logic. Existing core/adapter
splits and narrow store authority still apply. The historical first-wave
`shell only` bucket is not a new exemption for later work.

- `crates/crunch-build/src/action_result.rs` → `unrelated`
- `crates/crunch-build/src/artifact_auth.rs` → `unrelated`
- `crates/crunch-build/src/build_request.rs` → `unrelated`
- `crates/crunch-build/src/ca_input_resolution.rs` → `unrelated`
- `crates/crunch-build/src/ca_plan.rs` → `unrelated`
- `crates/crunch-build/src/causal_trace.rs` → `unrelated`
- `crates/crunch-build/src/compile_driver.rs` → `unrelated`
- `crates/crunch-build/src/dispatch_build_service.rs` → `unrelated`
- `crates/crunch-build/src/distributed.rs` → `unrelated`
- `crates/crunch-build/src/distributed/external_batch.rs` → `unrelated`
- `crates/crunch-build/src/distributed/remote_attempt_log.rs` → `unrelated`
- `crates/crunch-build/src/distributed/remote_failure_debug.rs` → `unrelated`
- `crates/crunch-build/src/distributed/remote_resources.rs` → `unrelated`
- `crates/crunch-build/src/distributed/remote_telemetry.rs` → `unrelated`
- `crates/crunch-build/src/distributed/remote_transfer.rs` → `unrelated`
- `crates/crunch-build/src/distributed/resource_policy.rs` → `unrelated`
- `crates/crunch-build/src/distributed/resource_quota_store.rs` → `unrelated`
- `crates/crunch-build/src/dynamic.rs` → `unrelated`
- `crates/crunch-build/src/dynamic_plan.rs` → `unrelated`
- `crates/crunch-build/src/dynamic_plan/slices.rs` → `unrelated`
- `crates/crunch-build/src/dynamic_plan/v2.rs` → `unrelated`
- `crates/crunch-build/src/dynamic_plan/wire.rs` → `unrelated`
- `crates/crunch-build/src/environment_policy.rs` → `unrelated`
- `crates/crunch-build/src/error.rs` → `unrelated`
- `crates/crunch-build/src/execution_profile.rs` → `unrelated`
- `crates/crunch-build/src/fetch_build_service.rs` → `unrelated`
- `crates/crunch-build/src/fetcher.rs` → `unrelated`
- `crates/crunch-build/src/finish_gates.rs` → `unrelated`
- `crates/crunch-build/src/fod.rs` → `unrelated`
- `crates/crunch-build/src/goal.rs` → `unrelated`
- `crates/crunch-build/src/hermeticity.rs` → `unrelated`
- `crates/crunch-build/src/lib.rs` → `unrelated`
- `crates/crunch-build/src/network_policy.rs` → `unrelated`
- `crates/crunch-build/src/orchestrate.rs` → `unrelated`
- `crates/crunch-build/src/plan_output_binding.rs` → `unrelated`
- `crates/crunch-build/src/registry.rs` → `unrelated`
- `crates/crunch-build/src/rewrite.rs` → `unrelated`
- `crates/crunch-build/src/scheduling.rs` → `unrelated`
- `crates/crunch-build/src/signing.rs` → `unrelated`
- `crates/crunch-build/src/trust_boundary_nominal.rs` → `unrelated`
- `crates/crunch-build/src/worker.rs` → `unrelated`
- `crates/crunch-build/src/workspace.rs` → `unrelated`
- `crates/crunch-build/src/workspace_shell.rs` → `unrelated`
- `crates/crunch-eval/src/lib.rs` → `unrelated`
- `crates/crunch-eval/src/session.rs` → `unrelated`
- `crates/crunch-eval/src/session/incremental.rs` → `unrelated`
- `crates/crunch-eval/src/stdlib.rs` → `unrelated`
- `crates/crunch-eval/src/watch_imports.rs` → `unrelated`
- `crates/crunch-glue/src/conversion_cache.rs` → `unrelated`
- `crates/crunch-glue/src/convert.rs` → `unrelated`
- `crates/crunch-glue/src/error.rs` → `unrelated`
- `crates/crunch-glue/src/lib.rs` → `unrelated`
- `crates/crunch-glue/src/types.rs` → `unrelated`
- `crates/crunch-nar/src/lib.rs` → `unrelated`
- `crates/crunch-pipeline/src/derivation_file.rs` → `unrelated`
- `crates/crunch-pipeline/src/evaluation_stream.rs` → `unrelated`
- `crates/crunch-pipeline/src/jobs_policy.rs` → `unrelated`
- `crates/crunch-pipeline/src/lib.rs` → `unrelated`
- `crates/crunch-pipeline/src/native_preflight.rs` → `unrelated`
- `crates/crunch-pipeline/src/watch.rs` → `unrelated`
- `crates/crunch-store/src/action_result.rs` → `unrelated`
- `crates/crunch-store/src/archive.rs` → `unrelated`
- `crates/crunch-store/src/backend.rs` → `unrelated`
- `crates/crunch-store/src/build_io.rs` → `unrelated`
- `crates/crunch-store/src/ca_mapping.rs` → `unrelated`
- `crates/crunch-store/src/capability.rs` → `unrelated`
- `crates/crunch-store/src/casita.rs` → `unrelated`
- `crates/crunch-store/src/completeness.rs` → `unrelated`
- `crates/crunch-store/src/composition.rs` → `unrelated`
- `crates/crunch-store/src/error.rs` → `unrelated`
- `crates/crunch-store/src/export.rs` → `unrelated`
- `crates/crunch-store/src/handle.rs` → `unrelated`
- `crates/crunch-store/src/http_closure.rs` → `unrelated`
- `crates/crunch-store/src/interest_store.rs` → `unrelated`
- `crates/crunch-store/src/layer.rs` → `unrelated`
- `crates/crunch-store/src/lib.rs` → `unrelated`
- `crates/crunch-store/src/metadata_cache.rs` → `unrelated`
- `crates/crunch-store/src/mutation_lock.rs` → `unrelated`
- `crates/crunch-store/src/nario.rs` → `unrelated`
- `crates/crunch-store/src/overlay.rs` → `unrelated`
- `crates/crunch-store/src/path_identity.rs` → `unrelated`
- `crates/crunch-store/src/provenance.rs` → `unrelated`
- `crates/crunch-store/src/publisher.rs` → `unrelated`
- `crates/crunch-store/src/pull.rs` → `unrelated`
- `crates/crunch-store/src/push.rs` → `unrelated`
- `crates/crunch-store/src/query.rs` → `unrelated`
- `crates/crunch-store/src/reference_scan.rs` → `unrelated`
- `crates/crunch-store/src/roots.rs` → `unrelated`

### Other specialized host and consumer families

Android extraction/native execution, hardware simulation, KernelScript,
SpaceWasm, WebAssembly materialization/verification and the Nix gateway each have
their own core/adapter contracts, separate from the canonical adopted set.
The listed files handle those families' host observations, protocol decoding,
process/tool stages, output materialization or consumer verification, with
family-specific models/checks. Their existing boundaries and explicitly limited
claims remain in force; calling them unrelated to this migration does not
certify those implementations or broaden their cohorts.

- `crates/crunch-android/src/authority.rs` → `unrelated`
- `crates/crunch-android/src/bin/admit.rs` → `unrelated`
- `crates/crunch-android/src/bin/apk.rs` → `unrelated`
- `crates/crunch-android/src/integration_tests/official_proof.rs` → `unrelated`
- `crates/crunch-android/src/lib.rs` → `unrelated`
- `crates/crunch-android/src/native.rs` → `unrelated`
- `crates/crunch-hardware-simulation/src/lib.rs` → `unrelated`
- `crates/crunch-kernelscript-adapter/src/lib.rs` → `unrelated`
- `crates/crunch-kernelscript-adapter/src/main.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/api.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/api/crypto.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/api/journal.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/lib.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/mutation.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/replay.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/server.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/store.rs` → `unrelated`
- `crates/crunch-spacewasm/src/lib.rs` → `unrelated`
- `crates/crunch-spacewasm/src/main.rs` → `unrelated`
- `crates/crunch-wasm-component/src/aot.rs` → `unrelated`
- `crates/crunch-wasm-component/src/error.rs` → `unrelated`
- `crates/crunch-wasm-component/src/files.rs` → `unrelated`
- `crates/crunch-wasm-component/src/lib.rs` → `unrelated`
- `crates/crunch-wasm-component/src/materialization.rs` → `unrelated`
- `crates/crunch-wasm-component/src/model.rs` → `unrelated`
- `crates/crunch-wasm-component/src/octet.rs` → `unrelated`
- `crates/crunch-wasm-component/src/pipeline.rs` → `unrelated`
- `crates/crunch-wasm-component/src/preflight.rs` → `unrelated`
- `crates/crunch-wasm-component/src/process.rs` → `unrelated`
- `crates/crunch-wasm-component/src/reporting.rs` → `unrelated`
- `crates/crunch-wasm-component/src/stages.rs` → `unrelated`
- `crates/crunch-wasm-component/src/toolchain.rs` → `unrelated`
- `crates/crunch-wasm-component/src/verification.rs` → `unrelated`
- `crates/crunch-wasm-component/src/wizer.rs` → `unrelated`
- `crates/mantle-wasm-consumer-verifier/src/contract.rs` → `unrelated`
- `crates/mantle-wasm-consumer-verifier/src/facade.rs` → `unrelated`
- `crates/mantle-wasm-consumer-verifier/src/lib.rs` → `unrelated`
- `crates/mantle-wasm-consumer-verifier/src/shell.rs` → `unrelated`

### Rust compiler/cache, vendor producer and differential shells

The Rust-cache façade and compiler-wrapper binaries own filesystem/castore,
socket/peer/framing, process, compile-driver/slot and result-publication effects
around their own Rust-cache decisions. The lock/vendor producer emits its own
bounded acquisition plan; the ordinary differential runner owns fixture/tool
execution, process containment and measured comparisons around its distinct
policy core. These are not first-wave attestation/project transforms or enrolled
builder cores, and no cache, compiler-correctness or equivalence result is
claimed by these source entries.

- `crates/crunch-rust-cache/src/cc.rs` → `unrelated`
- `crates/crunch-rust-cache/src/lib.rs` → `unrelated`
- `crates/crunch-rust-cache/src/shared.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/bin/mantle-cc-driver.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/bin/mantle-rust-cache-daemon.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/bin/mantle-rustc-manifest.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/bin/mantle-rustc-wrapper.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/cc.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/lib.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/slots.rs` → `unrelated`
- `crates/mantle-lock-vendor-producer/src/main.rs` → `unrelated`
- `crates/mantle-rust-differential/src/bin/wait-wrapper.rs` → `unrelated`
- `crates/mantle-rust-differential/src/cohorts.rs` → `unrelated`
- `crates/mantle-rust-differential/src/fixtures.rs` → `unrelated`
- `crates/mantle-rust-differential/src/lib.rs` → `unrelated`
- `crates/mantle-rust-differential/src/process.rs` → `unrelated`
- `crates/mantle-rust-differential/src/runner.rs` → `unrelated`
- `crates/mantle-rust-differential/src/waits.rs` → `unrelated`

### Root bootstrap, provider, fixed-point and source workflow families

These modules implement source-root/provider selection and capabilities,
bootstrap/StageX recipes and tool stages, source-built action execution,
fixed-point/resume/receipt/trust workflows, witness handoff and related measured
artifacts. Their bootstrap, source, application and release-core consumers do
not make the entire modules thin adapters: they also contain workflow-specific
decisions and evidence interpretation outside the first-wave extracted module
list. In particular, fixed-point receipt/trust-report and early native receipt
logic is not being certified business-logic-free. This ledger does not endorse
any inferred trust, bootstrap completeness or reproducibility result.

- `src/bootstrap.rs` → `unrelated`
- `src/bootstrap_parity.rs` → `unrelated`
- `src/bootstrap_source_root.rs` → `unrelated`
- `src/bootstrap_validate.rs` → `unrelated`
- `src/cargo_free_self_build.rs` → `unrelated`
- `src/early_native_row_receipt.rs` → `unrelated`
- `src/early_native_row_receipt_shell.rs` → `unrelated`
- `src/final_native_row_receipt_shell.rs` → `unrelated`
- `src/fresh_clone_fixed_point.rs` → `unrelated`
- `src/full_source_provider.rs` → `unrelated`
- `src/full_source_rust_binding.rs` → `unrelated`
- `src/full_source_rust_binding_shell.rs` → `unrelated`
- `src/native_toolchain_closure.rs` → `unrelated`
- `src/presentation/source_root_provider.rs` → `unrelated`
- `src/provider_output_publication.rs` → `unrelated`
- `src/rust_bootstrap_patch_plan.rs` → `unrelated`
- `src/rust_source_provider.rs` → `unrelated`
- `src/rust_source_provider_scratch.rs` → `unrelated`
- `src/self_build.rs` → `unrelated`
- `src/source_built_fixed_point.rs` → `unrelated`
- `src/source_built_fixed_point_action_trust.rs` → `unrelated`
- `src/source_built_fixed_point_cold.rs` → `unrelated`
- `src/source_built_fixed_point_dev_cache.rs` → `unrelated`
- `src/source_built_fixed_point_mantle_actions.rs` → `unrelated`
- `src/source_built_fixed_point_native_actions.rs` → `unrelated`
- `src/source_built_fixed_point_receipt.rs` → `unrelated`
- `src/source_built_fixed_point_resume.rs` → `unrelated`
- `src/source_built_fixed_point_resume_bundle.rs` → `unrelated`
- `src/source_built_fixed_point_rust_actions.rs` → `unrelated`
- `src/source_built_fixed_point_shell.rs` → `unrelated`
- `src/source_built_fixed_point_stagex_actions.rs` → `unrelated`
- `src/source_built_fixed_point_trust_report.rs` → `unrelated`
- `src/source_built_recipe.rs` → `unrelated`
- `src/source_built_recipe_exec.rs` → `unrelated`
- `src/source_root_capability.rs` → `unrelated`
- `src/source_root_manifest.rs` → `unrelated`
- `src/source_root_provider.rs` → `unrelated`
- `src/source_root_provider_flow.rs` → `unrelated`
- `src/source_toolchain_closure.rs` → `unrelated`
- `src/stagex_archive_core.rs` → `unrelated`
- `src/stagex_bash.rs` → `unrelated`
- `src/stagex_bash_full.rs` → `unrelated`
- `src/stagex_binutils.rs` → `unrelated`
- `src/stagex_bison.rs` → `unrelated`
- `src/stagex_bzip2.rs` → `unrelated`
- `src/stagex_command_capture.rs` → `unrelated`
- `src/stagex_coreutils.rs` → `unrelated`
- `src/stagex_diffutils.rs` → `unrelated`
- `src/stagex_flex.rs` → `unrelated`
- `src/stagex_gawk.rs` → `unrelated`
- `src/stagex_gnu_patch.rs` → `unrelated`
- `src/stagex_grep.rs` → `unrelated`
- `src/stagex_gzip.rs` → `unrelated`
- `src/stagex_m4.rs` → `unrelated`
- `src/stagex_make.rs` → `unrelated`
- `src/stagex_mes.rs` → `unrelated`
- `src/stagex_mes_lib.rs` → `unrelated`
- `src/stagex_mes_sources.rs` → `unrelated`
- `src/stagex_musl.rs` → `unrelated`
- `src/stagex_musl_native.rs` → `unrelated`
- `src/stagex_oyacc.rs` → `unrelated`
- `src/stagex_patch.rs` → `unrelated`
- `src/stagex_provider.rs` → `unrelated`
- `src/stagex_sed.rs` → `unrelated`
- `src/stagex_sources.rs` → `unrelated`
- `src/stagex_stage0.rs` → `unrelated`
- `src/stagex_stage0_full.rs` → `unrelated`
- `src/stagex_tar.rs` → `unrelated`
- `src/stagex_tcc_musl.rs` → `unrelated`
- `src/stagex_tcc_musl_prep.rs` → `unrelated`
- `src/stagex_tcc_musl_v2.rs` → `unrelated`
- `src/stagex_tcc_selfhost.rs` → `unrelated`
- `src/stagex_tinycc.rs` → `unrelated`
- `src/stagex_tinycc27.rs` → `unrelated`
- `src/stagex_transition.rs` → `unrelated`
- `src/witness_handoff.rs` → `unrelated`
- `src/witness_rebuild.rs` → `unrelated`

### Root remote orchestration and operator state families

These modules bind remote provider/endpoint/demand configuration, credentials
and secret state, transport/attempt logs, live workers, telemetry, transfer and
failure/debug observations to their remote/application contracts. Their
admission, state-machine and authority obligations are independent of the
first-wave attestation/project extraction; no live remote execution, recovery
or transport acceptance was performed for this review.

- `src/external_batch_dispatch.rs` → `unrelated`
- `src/presentation/remote_client.rs` → `unrelated`
- `src/remote_attempt_log_store.rs` → `unrelated`
- `src/remote_build.rs` → `unrelated`
- `src/remote_credential_state.rs` → `unrelated`
- `src/remote_credentials.rs` → `unrelated`
- `src/remote_failure_debug.rs` → `unrelated`
- `src/remote_failure_replay_identity.rs` → `unrelated`
- `src/remote_farm_config.rs` → `unrelated`
- `src/remote_nominal.rs` → `unrelated`
- `src/remote_service_secrets.rs` → `unrelated`
- `src/remote_telemetry_export.rs` → `unrelated`
- `src/remote_test_interruption.rs` → `unrelated`
- `src/remote_trace_context.rs` → `unrelated`
- `src/remote_transfer.rs` → `unrelated`

### Root foreign, OCI, ELF and Nix interoperability families

These paths own foreign derivation/graph/artifact projection, OCI layout and
registry transport, ELF symbol/fixup decisions and host execution, and Nix
import/producer/evidence boundaries. Files such as `elf_dynamic_fixup_core.rs`,
`elf_local_symbol_core.rs` and `nix_evidence_core.rs` contain distinct decision
logic despite living under std `src/`; they are not claimed to be adapter-only
or exempted from the functional-core architectural law. No extracted
first-wave schema/canonicalization/refresh algorithm is reassigned to them.

- `src/elf_dynamic_fixup_core.rs` → `unrelated`
- `src/elf_dynamic_fixup_shell.rs` → `unrelated`
- `src/elf_local_symbol_core.rs` → `unrelated`
- `src/elf_local_symbol_shell.rs` → `unrelated`
- `src/foreign_derivation_import.rs` → `unrelated`
- `src/foreign_executable_plan.rs` → `unrelated`
- `src/foreign_graph_compiler.rs` → `unrelated`
- `src/foreign_import_cmd.rs` → `unrelated`
- `src/foreign_provenance_audit.rs` → `unrelated`
- `src/foreign_realization.rs` → `unrelated`
- `src/foreign_realization_receipt.rs` → `unrelated`
- `src/foreign_realization_shell.rs` → `unrelated`
- `src/frontend_artifact_export.rs` → `unrelated`
- `src/frontend_artifact_spec.rs` → `unrelated`
- `src/frontend_artifact_store.rs` → `unrelated`
- `src/nix_derivation_adapter.rs` → `unrelated`
- `src/nix_evidence_core.rs` → `unrelated`
- `src/nix_gateway.rs` → `unrelated`
- `src/nix_producer.rs` → `unrelated`
- `src/nix_producer_shell.rs` → `unrelated`
- `src/oci_projection.rs` → `unrelated`
- `src/oci_projection_shell.rs` → `unrelated`
- `src/oci_registry.rs` → `unrelated`
- `src/oci_registry_shell.rs` → `unrelated`

### Remaining root composition, build, Cargo and presentation families

These paths cover the CLI composition roots and library exports; build/eval
commands, plans, logs and reports; Cargo acquisition/import/profile/build-bundle
and Rust-plan integration; store/artifact/profile/user-profile workflows;
operator/semantic-graph/refactor/presentation diagnostics; environment,
protected-process, file and signing helpers; and other source/bundle/receipt
adapters. `main.rs` and `lib.rs` wire consumers rather than owning the extracted
attestation/project algorithms. Their modules can contain distinct pure policy
and mixed application logic, and incidental use of adopted core DTOs does not
justify labeling a whole build/provider/operator module adapter-only. This is
migration-relative ownership classification, not global FCIS approval.

- `src/ambient_env.rs` → `unrelated`
- `src/artifact_cmd.rs` → `unrelated`
- `src/build_cmd.rs` → `unrelated`
- `src/build_correctness.rs` → `unrelated`
- `src/build_failure.rs` → `unrelated`
- `src/build_interchange.rs` → `unrelated`
- `src/build_log.rs` → `unrelated`
- `src/build_plan.rs` → `unrelated`
- `src/build_report.rs` → `unrelated`
- `src/cache_substitution.rs` → `unrelated`
- `src/cargo_acquisition.rs` → `unrelated`
- `src/cargo_build_bundle.rs` → `unrelated`
- `src/cargo_build_bundle_cmd.rs` → `unrelated`
- `src/cargo_import.rs` → `unrelated`
- `src/cargo_import_cli.rs` → `unrelated`
- `src/cargo_profile.rs` → `unrelated`
- `src/cargo_profile_manifest.rs` → `unrelated`
- `src/causal_trace_diagnostic.rs` → `unrelated`
- `src/child_exec.rs` → `unrelated`
- `src/command_input.rs` → `unrelated`
- `src/distributed_eval_assessment.rs` → `unrelated`
- `src/errors.rs` → `unrelated`
- `src/evaluation_stream_output.rs` → `unrelated`
- `src/evaluator_budget.rs` → `unrelated`
- `src/expression_scratch.rs` → `unrelated`
- `src/fix.rs` → `unrelated`
- `src/generated_file_check.rs` → `unrelated`
- `src/generated_file_write.rs` → `unrelated`
- `src/host_environment.rs` → `unrelated`
- `src/lib.rs` → `unrelated`
- `src/linux_rename.rs` → `unrelated`
- `src/log_cmd.rs` → `unrelated`
- `src/main.rs` → `unrelated`
- `src/mantlepkgs_adapter.rs` → `unrelated`
- `src/mantlepkgs_cmd.rs` → `unrelated`
- `src/mantlepkgs_version_cmd.rs` → `unrelated`
- `src/nickel_export.rs` → `unrelated`
- `src/nickel_export_core_adapter.rs` → `unrelated`
- `src/nix_free_demo_bundle.rs` → `unrelated`
- `src/nix_free_demo_cmd.rs` → `unrelated`
- `src/oci_projection/archive.rs` → `unrelated`
- `src/oci_projection/layout.rs` → `unrelated`
- `src/offline_cargo.rs` → `unrelated`
- `src/operator_contract.rs` → `unrelated`
- `src/operator_diagnostics.rs` → `unrelated`
- `src/picolibc_comparison.rs` → `unrelated`
- `src/portable_receipt.rs` → `unrelated`
- `src/presentation/diagnostics.rs` → `unrelated`
- `src/presentation/mod.rs` → `unrelated`
- `src/presentation/refactor.rs` → `unrelated`
- `src/presentation/reports.rs` → `unrelated`
- `src/presentation/runtime_fingerprint.rs` → `unrelated`
- `src/presentation/semantic_graph.rs` → `unrelated`
- `src/preserves_release_carrier.rs` → `unrelated`
- `src/process_env.rs` → `unrelated`
- `src/profile_cli.rs` → `unrelated`
- `src/project_build.rs` → `unrelated`
- `src/proof_clock_seccomp.rs` → `unrelated`
- `src/protected_exec.rs` → `unrelated`
- `src/protected_exec_seccomp.rs` → `unrelated`
- `src/realization_routing.rs` → `unrelated`
- `src/remote_build/demand.rs` → `unrelated`
- `src/remote_build/endpoint.rs` → `unrelated`
- `src/remote_build/live_state.rs` → `unrelated`
- `src/remote_build/live_state/daemon.rs` → `unrelated`
- `src/remote_build/live_state/producer.rs` → `unrelated`
- `src/run_binary_selection.rs` → `unrelated`
- `src/rust_plan.rs` → `unrelated`
- `src/rust_plan_execution.rs` → `unrelated`
- `src/rustc_dev_guide.rs` → `unrelated`
- `src/semantic_graph.rs` → `unrelated`
- `src/semantic_graph_cmd.rs` → `unrelated`
- `src/shared_rust_signing_key.rs` → `unrelated`
- `src/signing_key.rs` → `unrelated`
- `src/source_bundle.rs` → `unrelated`
- `src/source_observation.rs` → `unrelated`
- `src/store_cmd.rs` → `unrelated`
- `src/structured_refactor.rs` → `unrelated`
- `src/test_cli.rs` → `unrelated`
- `src/test_sentinel.rs` → `unrelated`
- `src/transcript_cmd.rs` → `unrelated`
- `src/trusted_keys.rs` → `unrelated`
- `src/user_profile_cmd.rs` → `unrelated`
- `src/user_profile_cmd/admission.rs` → `unrelated`
- `src/user_profile_cmd/publication.rs` → `unrelated`
- `src/vendor_source_manifest.rs` → `unrelated`
- `src/wasm_component_cmd.rs` → `unrelated`

### Tests, source fixtures, examples and benchmark-only consumers

These are integration/unit/architecture tests, their deliberately constrained
or invalid fixture crates, example-project source, benchmark/reference workloads
and test-support modules across the families above. Examples using delta or
other cores are consumers/fixtures, not replacement production implementations.
The classification does not say they ran or passed, that fixtures are accepted
production inputs, or that test-only logic is a new std production owner of the
extracted first-wave algorithms.

- `crates/crunch-action-result-core/tests/architecture.rs` → `unrelated`
- `crates/crunch-android-core/src/elf_tests.rs` → `unrelated`
- `crates/crunch-android-core/src/tests.rs` → `unrelated`
- `crates/crunch-android/src/integration_tests.rs` → `unrelated`
- `crates/crunch-android/src/tests.rs` → `unrelated`
- `crates/crunch-build/src/distributed/resource_policy_tests.rs` → `unrelated`
- `crates/crunch-build/src/distributed/resource_quota_store_tests.rs` → `unrelated`
- `crates/crunch-build/src/test_support.rs` → `unrelated`
- `crates/crunch-cmake/tests/artifacts.rs` → `unrelated`
- `crates/crunch-composition-core/tests/boundary.rs` → `unrelated`
- `crates/crunch-delta/examples/vectorcdc_candidate_compare.rs` → `unrelated`
- `crates/crunch-delta/examples/vectorcdc_corpus.rs` → `unrelated`
- `crates/crunch-delta/examples/vectorcdc_fastcdc_baseline.rs` → `unrelated`
- `crates/crunch-esp-idf/tests/nickel_contract.rs` → `unrelated`
- `crates/crunch-eval/examples/evaluation_baseline.rs` → `unrelated`
- `crates/crunch-eval/src/session/incremental/tests.rs` → `unrelated`
- `crates/crunch-eval/tests/nickel_cohort_guard.rs` → `unrelated`
- `crates/crunch-evaluation-stream-core/src/tests.rs` → `unrelated`
- `crates/crunch-glue/src/tests.rs` → `unrelated`
- `crates/crunch-hardware-simulation-core/src/tests.rs` → `unrelated`
- `crates/crunch-hardware-simulation/src/tests.rs` → `unrelated`
- `crates/crunch-kernelscript-adapter/src/tests.rs` → `unrelated`
- `crates/crunch-kernelscript-core/src/tests.rs` → `unrelated`
- `crates/crunch-nar/tests/parity.rs` → `unrelated`
- `crates/crunch-nix-gateway/src/api/tests.rs` → `unrelated`
- `crates/crunch-nix-gateway/tests/nix_path_info.rs` → `unrelated`
- `crates/crunch-nix-gateway/tests/ticket_envelope.rs` → `unrelated`
- `crates/crunch-pipeline/tests/integration_build.rs` → `unrelated`
- `crates/crunch-remote-core/tests/authority.rs` → `unrelated`
- `crates/crunch-remote-core/tests/fixtures/architecture/host-filesystem/src/lib.rs` → `unrelated`
- `crates/crunch-remote-core/tests/fixtures/architecture/host-network/src/lib.rs` → `unrelated`
- `crates/crunch-remote-core/tests/fixtures/architecture/host-process/src/lib.rs` → `unrelated`
- `crates/crunch-remote-core/tests/fixtures/architecture/src/lib.rs` → `unrelated`
- `crates/crunch-rust-cache-core/tests/architecture.rs` → `unrelated`
- `crates/crunch-service-readiness-core/src/tests.rs` → `unrelated`
- `crates/crunch-source-core/tests/ingest_fixtures.rs` → `unrelated`
- `crates/crunch-source-core/tests/source_observation_fixtures.rs` → `unrelated`
- `crates/crunch-spacewasm-core/src/tests.rs` → `unrelated`
- `crates/crunch-spacewasm-core/tests/run_record_fixtures.rs` → `unrelated`
- `crates/crunch-spacewasm-core/tests/stable_report_contract_parity.rs` → `unrelated`
- `crates/crunch-spacewasm-core/tests/stable_report_fixtures.rs` → `unrelated`
- `crates/crunch-spacewasm/src/tests.rs` → `unrelated`
- `crates/crunch-store/tests/authority_source_policy.rs` → `unrelated`
- `crates/crunch-test/src/tests.rs` → `unrelated`
- `crates/crunch-verification-bridge/tests/boundaries.rs` → `unrelated`
- `crates/crunch-wasm-component-core/src/tests.rs` → `unrelated`
- `crates/crunch-wasm-component/src/process_tests.rs` → `unrelated`
- `crates/crunch-wasm-component/src/toolchain_tests.rs` → `unrelated`
- `crates/crunch-watch-core/src/tests.rs` → `unrelated`
- `crates/mantle-application-contract/tests/contract_fixtures.rs` → `unrelated`
- `crates/mantle-application-contract/tests/port_fixtures.rs` → `unrelated`
- `crates/mantle-build-contract/examples/emit-fixtures.rs` → `unrelated`
- `crates/mantle-build-contract/src/tests.rs` → `unrelated`
- `crates/mantle-build-contract/tests/producer_fixtures.rs` → `unrelated`
- `crates/mantle-cargo-build-bundle-core/tests/bundle.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/tests/build_script_lowering.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/tests/helper.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/tests/lowering.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/tests/producer.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/tests/source_nar.rs` → `unrelated`
- `crates/mantle-rust-differential-core/tests/evidence.rs` → `unrelated`
- `crates/mantle-rust-plan-app/tests/application_fixtures.rs` → `unrelated`
- `crates/mantle-rust-plan-core/tests/build_script_fixtures.rs` → `unrelated`
- `crates/mantle-rust-plan-core/tests/plan_fixtures.rs` → `unrelated`
- `crates/mantle-wasm-consumer-verifier/tests/consumer_fixtures.rs` → `unrelated`
- `examples/benchmark_compare.rs` → `unrelated`
- `examples/benchmark_eval_backends.rs` → `unrelated`
- `examples/benchmark_lazy_eval.rs` → `unrelated`
- `examples/benchmark_scheduler_priority.rs` → `unrelated`
- `examples/benchmark_support.rs` → `unrelated`
- `examples/distributed_eval_assess.rs` → `unrelated`
- `examples/hardware_simulation_plan.rs` → `unrelated`
- `examples/picolibc_compare.rs` → `unrelated`
- `examples/projects/cargo-import-offline/ambiguous-workspace/app-a/src/main.rs` → `unrelated`
- `examples/projects/cargo-import-offline/ambiguous-workspace/app-b/src/main.rs` → `unrelated`
- `examples/projects/cargo-import-offline/workspace/dep/src/lib.rs` → `unrelated`
- `examples/projects/cargo-import-offline/workspace/src/main.rs` → `unrelated`
- `examples/projects/delta-substitution/demo.rs` → `unrelated`
- `examples/projects/release-witness-handoff/demo.rs` → `unrelated`
- `examples/projects/rust-workspace/greeting/src/lib.rs` → `unrelated`
- `examples/projects/rust-workspace/workspace-app/src/main.rs` → `unrelated`
- `examples/projects/schema-codegen/app/src/main.rs` → `unrelated`
- `examples/projects/shared-action-result-roundtrip/publish.rs` → `unrelated`
- `examples/projects/wasm-component-hello/source/src/lib.rs` → `unrelated`
- `examples/rust_compatibility_rail.rs` → `unrelated`
- `src/machine_contract_producer_tests.rs` → `unrelated`
- `src/oci_projection/tests.rs` → `unrelated`
- `src/remote_build/tests/external_batch_hardware_tests.rs` → `unrelated`
- `src/user_profile_cmd/tests.rs` → `unrelated`
- `tests/android_apk.rs` → `unrelated`
- `tests/android_sources.rs` → `unrelated`
- `tests/android_sources_offline.rs` → `unrelated`
- `tests/benchmark_harness.rs` → `unrelated`
- `tests/bootstrap_eval.rs` → `unrelated`
- `tests/bootstrap_parity_cli.rs` → `unrelated`
- `tests/bootstrap_validate_cli.rs` → `unrelated`
- `tests/cargo_free_self_build_cli.rs` → `unrelated`
- `tests/cargo_import_cli.rs` → `unrelated`
- `tests/causal_build_trace.rs` → `unrelated`
- `tests/composition_root_cli.rs` → `unrelated`
- `tests/elf_dynamic_reference_scanner.rs` → `unrelated`
- `tests/elf_dynamic_relocation.rs` → `unrelated`
- `tests/evaluator_budget_cli.rs` → `unrelated`
- `tests/example_projects.rs` → `unrelated`
- `tests/examples_build.rs` → `unrelated`
- `tests/examples_eval.rs` → `unrelated`
- `tests/examples_inventory.rs` → `unrelated`
- `tests/examples_workflow_gallery.rs` → `unrelated`
- `tests/finish_gates.rs` → `unrelated`
- `tests/fixtures/android-native-minimal/rust/build.rs` → `unrelated`
- `tests/fixtures/android-native-minimal/rust/macro/src/lib.rs` → `unrelated`
- `tests/fixtures/android-native-minimal/rust/src/lib.rs` → `unrelated`
- `tests/fixtures/unified-test/fixture.rs` → `unrelated`
- `tests/fixtures/verification-bridge/octet/proofs/aeneas/source/src/lib.rs` → `unrelated`
- `tests/fixtures/verification-bridge/octet/src/lib.rs` → `unrelated`
- `tests/fixtures/verification-bridge/trellis/src/arithmetic.rs` → `unrelated`
- `tests/foreign_import_cli.rs` → `unrelated`
- `tests/freshness_offline_rail.rs` → `unrelated`
- `tests/identity_cli.rs` → `unrelated`
- `tests/integration.rs` → `unrelated`
- `tests/integration_build.rs` → `unrelated`
- `tests/kernel_bundle_oci_cli.rs` → `unrelated`
- `tests/kernel_bundle_oci_registry_cli.rs` → `unrelated`
- `tests/kernelscript_experiment.rs` → `unrelated`
- `tests/lock_importer_offline_rail.rs` → `unrelated`
- `tests/machine_schema_contracts.rs` → `unrelated`
- `tests/nix_free_demo_cli.rs` → `unrelated`
- `tests/offline_build_runbook_docs.rs` → `unrelated`
- `tests/offline_cargo_project.rs` → `unrelated`
- `tests/operator_diagnostics.rs` → `unrelated`
- `tests/pin_import_cli.rs` → `unrelated`
- `tests/project_build_smoke.rs` → `unrelated`
- `tests/release_cli.rs` → `unrelated`
- `tests/remote_credentials_cli.rs` → `unrelated`
- `tests/remote_hexagon_v2_parity.rs` → `unrelated`
- `tests/remote_rail_offline_rail.rs` → `unrelated`
- `tests/remote_stdio_cli.rs` → `unrelated`
- `tests/remote_transfer_production.rs` → `unrelated`
- `tests/removed_system_cli.rs` → `unrelated`
- `tests/retention_interest_cli.rs` → `unrelated`
- `tests/retention_offline_rail.rs` → `unrelated`
- `tests/rust_compatibility_rail.rs` → `unrelated`
- `tests/rust_plan_cli.rs` → `unrelated`
- `tests/scheduling_policy.rs` → `unrelated`
- `tests/self_hosting.rs` → `unrelated`
- `tests/smoke.rs` → `unrelated`
- `tests/source_bundle_cli.rs` → `unrelated`
- `tests/source_bundle_hydration_cli.rs` → `unrelated`
- `tests/stdlib_tests.rs` → `unrelated`
- `tests/store_archive_cli.rs` → `unrelated`
- `tests/store_gc_cli.rs` → `unrelated`
- `tests/support/distribution_registry.rs` → `unrelated`
- `tests/support/mod.rs` → `unrelated`
- `tests/support/oci_registry.rs` → `unrelated`
- `tests/transactional_reconciliation_pilot.rs` → `unrelated`
- `tests/transcript_cli.rs` → `unrelated`
- `tests/trust_policy_offline_rail.rs` → `unrelated`
- `tests/wasm_component_cli.rs` → `unrelated`

Supplement reviewer: OwnershipReviewCutover (source inspection and path metadata only).

## History-derived and prospective source ownership supplement (2026-10-25)

This supplement extends the path ledger to the paths committed or prospectively
touched since the 2026-10-18 supplement. It preserves the first-wave scope, the
review verdict and that supplement's meanings of `adapter-only` and `unrelated`.
It does not change the checker, its history derivation, the canonical inventory
or any policy.

### Derivation and source-review basis (2026-10-25)

The derivation called `touched_std_workspace_source_files`,
`required_std_adapter_files` and `ownership_review_has_classification` from
`scripts/no_std_core_checks.py` read-only. The history roots and diff base are
unchanged: first history revision `6e47f8725126943b26335abd6d1a6df4606fa3e8`,
parent `00190e4d1096905bf244d6aadf1664c7fd822625`. At reviewed HEAD
`0acfdf2545eba138304de99f6d5c92e8c2d79477` the predicate derives 869 historical
paths, which include all 20 required adapter paths. The existing exact-syntax
ledger covered 751 of them, leaving 118 historical paths without a
classification. The ownership step of `scripts/check-no-std-core.sh` reported
exactly these 118 paths in the 2026-10-25 final-gate run, the first run of that
cycle to pass the API-shape step.

Prospective coverage applies the same predicate and legacy exclusions to tracked
worktree changes and untracked, non-ignored Rust sources present during this
review: 123 eligible paths, 7 of them outside the historical union and
unclassified. The 125 rows below cover the 118 historical and those 7
prospective paths, all `unrelated`. With the 755 earlier exact rows the ledger
has 880 rows. This is a metadata snapshot, not a waiver for future source paths.

Family decisions use the 2026-10-18 basis: the adopted inventory, crate-root and
module ownership, module documentation and targeted inspection. Two read-only
scans bound the review. None of the 125 files names a crate listed in
`validation/adopted-core-inventory.toml`, and none defines a function, type or
trait named after any adopted core's required export. No first-wave moved
algorithm was identified as reimplemented in std. No gate, build, test, lint,
smoke, real-tool or remote workflow was run for this supplement.

### Root remote orchestration and operator state families (2026-10-25)

These `src/remote_build` modules decode registered UCAN filter caveats, fence
short coordinator saves, load operator-local build and store-source endpoint
names, resolve endpoint names on the coordination socket, keep gateway
identities in the coordinator snapshot, bridge remote application observations,
and account exact-child worker usage through a private executor child. Their
admission, authority, fencing and accounting obligations belong to the remote
and application contracts, independent of the first-wave extraction. No live
remote execution, transfer or recovery acceptance was performed for this review.

- `src/remote_build/authority_caveats.rs` → `unrelated`
- `src/remote_build/coordinator_merge.rs` → `unrelated`
- `src/remote_build/endpoint_config.rs` → `unrelated`
- `src/remote_build/endpoint_service.rs` → `unrelated`
- `src/remote_build/gateway.rs` → `unrelated`
- `src/remote_build/operation.rs` → `unrelated`
- `src/remote_build/usage.rs` → `unrelated`
- `src/remote_build/usage_child.rs` → `unrelated`

### Nix remote service gateway host families (2026-10-25)

These modules adapt Mantle's coordinator and ordinary remote execution/import
path to a private API journal, load owner-private production authority, drive
the local quota-filesystem fault fixture, project facts into Valence's
build-service evidence profile, serve operation-bound Nix import and attenuated
store-read endpoints, and host operator-configured resource selection and
durable quotas. Basalt admission, signed-store verification, journal
reconciliation and quota policy remain the gateway's own obligations. No hosted
service, VM, remote client or Valence acceptance was exercised for this review.

- `src/nix_gateway/api_host.rs` → `unrelated`
- `src/nix_gateway/authority.rs` → `unrelated`
- `src/nix_gateway/campaign.rs` → `unrelated`
- `src/nix_gateway/evidence.rs` → `unrelated`
- `src/nix_gateway/native.rs` → `unrelated`
- `src/nix_gateway/resource_host.rs` → `unrelated`

### Root bootstrap, provider, fixed-point and source workflow families (2026-10-25)

These modules run bootstrap capabilities with independent output verification
and compose bootstrap operations; drive the source-built fixed point's
controller, authoring, planning and verifier self-test; author the source-only
Mantle, original native GNU, StageX and selected Rust provider program routes;
and declare and run source recipe controllers through the common admitted step
runner. The Cargo lockfile and manifest reading in
`src/source_built_rust_resolve.rs` selects Rust provider packages from
authenticated release archives; it is not the Mantle project manifest and lock
handling that the first wave moved to `crunch-project-core`. Classification
makes no bootstrap, fixed-point, self-hosting, Cargo-free or release claim.

- `src/bootstrap_effect_port.rs` → `unrelated`
- `src/bootstrap_operation.rs` → `unrelated`
- `src/source_built_fixed_point_action_controller.rs` → `unrelated`
- `src/source_built_fixed_point_authoring.rs` → `unrelated`
- `src/source_built_fixed_point_planning.rs` → `unrelated`
- `src/source_built_fixed_point_verifier_self_test.rs` → `unrelated`
- `src/source_built_mantle_programs.rs` → `unrelated`
- `src/source_built_mantle_programs_aws.rs` → `unrelated`
- `src/source_built_mantle_programs_aws_helper.rs` → `unrelated`
- `src/source_built_mantle_programs_compression.rs` → `unrelated`
- `src/source_built_mantle_programs_graph.rs` → `unrelated`
- `src/source_built_mantle_programs_metadata.rs` → `unrelated`
- `src/source_built_mantle_programs_metadata_helper.rs` → `unrelated`
- `src/source_built_mantle_programs_native.rs` → `unrelated`
- `src/source_built_mantle_programs_native_helper.rs` → `unrelated`
- `src/source_built_mantle_programs_native_request.rs` → `unrelated`
- `src/source_built_mantle_programs_native_schedules.rs` → `unrelated`
- `src/source_built_mantle_programs_rust_probes.rs` → `unrelated`
- `src/source_built_mantle_programs_rust_probes_selected.rs` → `unrelated`
- `src/source_built_native_programs.rs` → `unrelated`
- `src/source_built_native_programs_early_gnu.rs` → `unrelated`
- `src/source_built_native_programs_early_gnu_generator_base.rs` → `unrelated`
- `src/source_built_native_programs_early_gnu_link_seams.rs` → `unrelated`
- `src/source_built_native_programs_early_gnu_originals.rs` → `unrelated`
- `src/source_built_native_programs_early_gnu_perl_child.rs` → `unrelated`
- `src/source_built_native_programs_early_gnu_tcc.rs` → `unrelated`
- `src/source_built_native_programs_final_gnu.rs` → `unrelated`
- `src/source_built_native_programs_final_gnu_generators.rs` → `unrelated`
- `src/source_built_native_programs_final_gnu_nodes.rs` → `unrelated`
- `src/source_built_native_programs_host.rs` → `unrelated`
- `src/source_built_native_programs_host_bootstrap.rs` → `unrelated`
- `src/source_built_native_programs_host_controller.rs` → `unrelated`
- `src/source_built_native_programs_host_original.rs` → `unrelated`
- `src/source_built_native_programs_host_packages.rs` → `unrelated`
- `src/source_built_native_programs_host_root.rs` → `unrelated`
- `src/source_built_native_programs_phases.rs` → `unrelated`
- `src/source_built_program_authoring.rs` → `unrelated`
- `src/source_built_recipe_command_state.rs` → `unrelated`
- `src/source_built_recipe_controller.rs` → `unrelated`
- `src/source_built_recipe_exec_controller.rs` → `unrelated`
- `src/source_built_recipe_exec_descriptor_transport.rs` → `unrelated`
- `src/source_built_recipe_exec_scopes.rs` → `unrelated`
- `src/source_built_rust_programs.rs` → `unrelated`
- `src/source_built_rust_programs_llvm.rs` → `unrelated`
- `src/source_built_rust_programs_macros.rs` → `unrelated`
- `src/source_built_rust_programs_minicargo.rs` → `unrelated`
- `src/source_built_rust_programs_minicargo_resolve.rs` → `unrelated`
- `src/source_built_rust_programs_mrustc.rs` → `unrelated`
- `src/source_built_rust_programs_patches.rs` → `unrelated`
- `src/source_built_rust_programs_phase.rs` → `unrelated`
- `src/source_built_rust_programs_runtime.rs` → `unrelated`
- `src/source_built_rust_programs_successors.rs` → `unrelated`
- `src/source_built_rust_programs_successors_cc.rs` → `unrelated`
- `src/source_built_rust_programs_successors_graph.rs` → `unrelated`
- `src/source_built_rust_programs_successors_helper.rs` → `unrelated`
- `src/source_built_rust_programs_successors_invocation.rs` → `unrelated`
- `src/source_built_rust_programs_successors_macro.rs` → `unrelated`
- `src/source_built_rust_programs_successors_native.rs` → `unrelated`
- `src/source_built_rust_programs_successors_native_helper.rs` → `unrelated`
- `src/source_built_rust_programs_successors_process.rs` → `unrelated`
- `src/source_built_rust_programs_successors_profile.rs` → `unrelated`
- `src/source_built_rust_programs_successors_std_process.rs` → `unrelated`
- `src/source_built_rust_programs_successors_wrappers.rs` → `unrelated`
- `src/source_built_rust_programs_translation.rs` → `unrelated`
- `src/source_built_rust_resolve.rs` → `unrelated`
- `src/source_built_stagex_programs.rs` → `unrelated`
- `src/source_built_stagex_programs_binutils.rs` → `unrelated`
- `src/source_built_stagex_programs_binutils_dispatch.rs` → `unrelated`
- `src/source_built_stagex_programs_binutils_ldscripts.rs` → `unrelated`
- `src/source_built_stagex_programs_binutils_runtime.rs` → `unrelated`
- `src/source_built_stagex_programs_binutils_shellvm.rs` → `unrelated`
- `src/source_built_stagex_programs_binutils_writer.rs` → `unrelated`
- `src/source_built_stagex_programs_controller.rs` → `unrelated`
- `src/source_built_stagex_programs_early.rs` → `unrelated`
- `src/source_built_stagex_programs_generators.rs` → `unrelated`
- `src/source_built_stagex_programs_late.rs` → `unrelated`
- `src/source_built_stagex_programs_middle.rs` → `unrelated`
- `src/source_built_stagex_programs_publication.rs` → `unrelated`

### Remaining root composition, build, evaluation and project-build families (2026-10-25)

`src/build_application.rs` prepares project-backed realization through ports,
`src/build_observation.rs` classifies build capability observations
independently of presentation, `src/evaluation_cmd.rs` runs the evaluation
application and verifies its output and report evidence before rendering, and
`src/project_build/authority.rs` receives signed, owner-bound project
declarations against an operator-configured canonical root through the gateway
verifier. None of them references `crunch-project-core` or redefines its
exports; project manifest and lock handling stays with the reviewed project
adapters.

- `src/build_application.rs` → `unrelated`
- `src/build_observation.rs` → `unrelated`
- `src/evaluation_cmd.rs` → `unrelated`
- `src/project_build/authority.rs` → `unrelated`

### Application contracts and effect-sequencing families (2026-10-25)

These `mantle-application-contract` modules declare the bootstrap capability
boundaries and their shared effect envelope, the gateway CLI operation
boundaries, and the Rust toolchain, bundle, test and explicit-profile operation
boundaries. They are application contracts consumed by root shells, not adopted
functional cores.

- `crates/mantle-application-contract/src/bootstrap_effects.rs` → `unrelated`
- `crates/mantle-application-contract/src/gateway_operations.rs` → `unrelated`
- `crates/mantle-application-contract/src/rust_operations.rs` → `unrelated`

### Other decision and contract families outside the adopted inventory (2026-10-25)

`crates/crunch-remote-core/src/trellis_projection.rs` is a finite,
equality-preserving projection onto the Trellis fenced-attempt kernel: a
consumer mapping, not a second admission model. `crunch-remote-core` is outside
the adopted inventory, and no Trellis proof or Valence acceptance is claimed.

- `crates/crunch-remote-core/src/trellis_projection.rs` → `unrelated`

### Build, evaluator, conversion, pipeline and store runtime families (2026-10-25)

`crates/crunch-build/src/rust_waits.rs` attaches local Cargo wait observation
after identity-bearing request construction,
`crates/crunch-store/src/fault_scope.rs` gives tests thread-scoped ownership of
the pinned fault-injection counter, and
`crates/crunch-store/src/source_admission.rs` admits declared read sources
against operator-owned endpoint bindings. They are build and store runtime
adapters with their own authority and bounding obligations.

- `crates/crunch-build/src/rust_waits.rs` → `unrelated`
- `crates/crunch-store/src/fault_scope.rs` → `unrelated`
- `crates/crunch-store/src/source_admission.rs` → `unrelated`

### Rust compiler/cache, vendor producer and differential shells (2026-10-25)

The `crunch-rustc-wrapper` wait wrapper binary, observed-process supervision
(including descendant exec observation) and wait-diagnostic collection, and the
`mantle-rust-differential` entry point, negative campaign and serial
direct-rustc baseline observe and compare compiler executions. Their decisions
live in `mantle-rust-differential-core` and the wrapper's own contracts, outside
the adopted inventory. No differential qualification, speedup or
compiler-correctness claim follows from this classification.

- `crates/crunch-rustc-wrapper/src/bin/mantle-rust-wait-wrapper.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/observed_process.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/observed_process/exec.rs` → `unrelated`
- `crates/crunch-rustc-wrapper/src/waits.rs` → `unrelated`
- `crates/mantle-rust-differential/src/main.rs` → `unrelated`
- `crates/mantle-rust-differential/src/negative.rs` → `unrelated`
- `crates/mantle-rust-differential/src/serial.rs` → `unrelated`

### Tests, source fixtures, examples and benchmark-only consumers (2026-10-25)

These are unit and integration tests, test-support modules, a resource-benchmark
fixture workload and the resource-policy replay example for the families above.
The GPU artifact test drives the enrolled `crunch-gpu` std adapter's public
inspection over a synthetic object, as the CMake artifact test does for its
adapter. Classification does not say they ran or passed, or that test-only
logic is a std production owner of an extracted algorithm.

- `crates/crunch-gpu/tests/artifact.rs` → `unrelated`
- `crates/crunch-remote-core/src/trellis_projection_tests.rs` → `unrelated`
- `crates/mantle-cargo-unit-plan/tests/support/mod.rs` → `unrelated`
- `examples/resource_policy_replay.rs` → `unrelated`
- `src/nix_gateway/api_host_tests.rs` → `unrelated`
- `src/nix_gateway/authority/tests.rs` → `unrelated`
- `src/nix_gateway/evidence/tests.rs` → `unrelated`
- `src/nix_gateway/resource_host_tests.rs` → `unrelated`
- `src/remote_build/tests/coordinator_merge_tests.rs` → `unrelated`
- `src/remote_build/tests/endpoint_binding_tests.rs` → `unrelated`
- `src/remote_build/tests/source_binding_tests.rs` → `unrelated`
- `src/source_built_fixed_point_action_controller_tests.rs` → `unrelated`
- `src/source_built_recipe_exec_tests.rs` → `unrelated`
- `tests/fixtures/resource-benchmark/workload.rs` → `unrelated`
- `tests/support/remote_endpoint_binding.rs` → `unrelated`

Supplement reviewer: omp session 2026-10-25 (source inspection and path metadata only).
