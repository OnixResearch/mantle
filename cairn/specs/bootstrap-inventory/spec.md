# Bootstrap Inventory Specification

## Purpose

Defines the `bootstrap-inventory` capability.

## Requirements

### Requirement: Bootstrap blocker inventory signal

r[bootstrap_inventory.blocker_signal] Mantle MUST distinguish successful bootstrap blocker inventory report generation from clean-baseline enforcement failure, while preserving fail-closed promotion-claim and unsuppressed-blocker detection.

#### Scenario: report-only inventory succeeds with remaining blockers

GIVEN bootstrap-critical sources still contain known blocker markers
WHEN an operator runs the blocker inventory in report-only mode
THEN Mantle MUST write valid JSON and Markdown reports and exit successfully if report generation succeeds
AND the report MUST state that blockers remain without presenting the repository as clean.

#### Scenario: enforcement fails on blockers or promotion claims

GIVEN unsuppressed bootstrap blockers or bootstrap promotion claims are present
WHEN the inventory runs in enforcement mode
THEN Mantle MUST fail closed with a deterministic diagnostic
AND it MUST preserve enough report output for the operator to identify the blocker class and source.

#### Scenario: checked metadata is separated from actionable findings

GIVEN checked evidence metadata contains bridge, placeholder, or frontier wording that is covered by a durable suppression reason
WHEN the inventory report is rendered
THEN Mantle MUST keep that metadata auditable in a separate suppressed or informational section
AND it MUST NOT let metadata-only records obscure the primary actionable source-finding list.

#### Scenario: suppression does not hide live source blockers

GIVEN a bootstrap source file contains an unsuppressed blocker marker with wording similar to a suppressed evidence metadata record
WHEN the inventory classifies findings
THEN the source marker MUST remain counted as actionable
AND the report MUST NOT classify it as metadata-only without an explicit evidence-backed suppression.

### Requirement: Bootstrap gauntlet inventory pressure

r[bootstrap_inventory.bootstrap_gauntlet_inventory_pressure] Mantle bootstrap pressure profiles that claim no-host-tools or protected-exec coverage MUST bind every permitted protected-phase executable to a declared inventory entry.

#### Scenario: declared executable inventory is complete

GIVEN a bootstrap pressure profile enters a protected phase
WHEN protected exec observes an executable path
THEN the path MUST match a host prerequisite, pinned fetched artifact, or Mantle-built output declared in the profile inventory
AND the audit report MUST bind the inventory digest and observed executable digest.

#### Scenario: missing inventory entry fails closed

GIVEN protected exec observes an executable that is not declared by the active profile inventory
WHEN the bootstrap pressure profile evaluates the phase
THEN the profile MUST fail closed before recording no-host-tools success
AND the diagnostic MUST identify the undeclared executable class without promoting the profile.

### Requirement: Bootstrap inputs are source-bundle admissible

r[bootstrap_inventory.offline_bootstrap_source_bundles] Mantle MUST provide an offline bootstrap source-bundle profile that can describe, import, pin, and preflight the source/input material required by selected bootstrap and self-build workflows. The profile MUST bind provider archive identity, provider metadata, bootstrap source archives, Mantle source tree identity, vendored Cargo input identity when applicable, toolchain/source-root records, logical store prefix, fixed-output hashes, and proof-mode requirements before an offline bootstrap command can consume those inputs.

#### Scenario: complete bootstrap bundle permits offline source acquisition

GIVEN an operator imports and pins a bootstrap source bundle whose records match the selected bootstrap profile
AND the profile includes every source, provider, toolchain, source tree, and vendored input required by that mode
WHEN Mantle runs bootstrap preflight or an offline bootstrap command
THEN Mantle MAY use the imported source state instead of live network source fetches
AND the report MUST bind the profile digest and source-state digest used for the decision.

#### Scenario: incomplete bootstrap bundle fails closed

GIVEN a bootstrap source bundle is missing a required provider archive, provider manifest, source archive, source tree, vendored Cargo input, toolchain source-root record, or proof input for the selected mode
WHEN Mantle evaluates offline bootstrap readiness
THEN Mantle MUST reject the profile before bootstrap execution
AND diagnostics MUST name the missing or stale record class.

#### Scenario: provider metadata mismatch blocks offline bootstrap

GIVEN imported source state contains provider material with a wrong provider kind, unsupported metadata schema, stale fixed-output hash, mismatched logical store prefix, missing reduced-provider provenance, or normalized seed contract mismatch
WHEN Mantle validates the offline bootstrap profile
THEN Mantle MUST fail closed before consuming the provider
AND it MUST NOT downgrade to an online fetch or a broader bootstrap claim silently.

#### Scenario: source-bundle readiness is not bootstrap proof

GIVEN Mantle reports an offline bootstrap source bundle as ready
WHEN an evidence file, task, documentation page, or status reply cites that readiness
THEN the claim MUST be limited to bootstrap source/input material being locally available and identity-matched
AND it MUST NOT claim provider trust removal, compiler correctness, self-build success, release reproducibility, or full bootstrap correctness without separate proof evidence.

### Requirement: Self-build source staging closes over current compile inputs

r[bootstrap_inventory.self_build_source_closure] Mantle MUST validate one explicit checkout-local Cargo directory source against the locked registry/git package graph and Cargo checksum metadata, stage every fixed build-required top-level source root, and require current fixed-point evidence before reporting self-build success.

#### Scenario: Complete vendor input resolves without ambient Cargo state

GIVEN `vendor-deps/` was generated from the current locked graph and `.cargo/vendor-config.toml` selects that directory source
WHEN locked offline Cargo metadata runs with an empty `CARGO_HOME`, offline network policy, and no ambient registry or git cache
THEN every locked registry/git package MUST resolve from the explicit directory source
AND missing packages, stale package checksums, stale file checksums, extra packages, or unsupported source replacements MUST fail closed before bootstrap.

#### Scenario: Staged source includes compile-time policy bytes

GIVEN a tracked workspace crate consumes generated policy bytes from the top-level `config/` root during compilation
WHEN Mantle stages the fixed source tree for self-build
THEN the exact policy bytes MUST be present at the same relative path in staged source
AND unrelated roots such as `target/`, arbitrary scratch files, and private `.pi` content MUST remain excluded.

#### Scenario: Bootstrap target preserves no-clobber publication

GIVEN the bootstrap Rust target uses Linux with libc bindings that do not expose the `renameat2` function symbol
WHEN Mantle compiles and exercises OCI, release, attempt-log, or remote-failure publication
THEN the shared Linux shell MUST invoke the kernel no-replace rename operation without depending on that function binding
AND an existing destination MUST remain unchanged together with the unpublished source.

#### Scenario: Installed runtime configuration does not retain staged source identity

GIVEN a production Nickel configuration loader needs Mantle's standard library
WHEN Mantle compiles inside a transient staged source root
THEN runtime import resolution MUST use a discovered source stdlib or the embedded stdlib materialization path
AND the final installed binary MUST NOT retain the transient staged source root as runtime data.

#### Scenario: Proof encounters an intermediate frontier

GIVEN offline metadata, vendor checksum validation, proof preflight, bootstrap tools, or stage1 compilation succeeds
WHEN a later fixed-point stage fails or its evidence is incomplete
THEN Mantle MUST report the exact current blocker and diagnostics instead of self-build success
AND only a current proof bundle with admitted stage1/stage2 equality MAY support the bounded fixed-point claim.

#### Scenario: Fixed-point evidence remains narrowly scoped

GIVEN the current self-build proof succeeds from the repaired source closure
WHEN operators or maintainers report that result
THEN they MUST identify the proof mode, source/vendor boundary, selected input transport, bundle path, and stage1/stage2 equality evidence
AND they MUST NOT infer compiler correctness, seed trust removal, fresh-clone offline completeness, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

### Requirement: Fresh clones hydrate explicit self-build inputs from a verified bundle

r[bootstrap_inventory.fresh_clone_source_hydration] Mantle MUST provide a bounded offline hydration workflow that reconstructs the ignored Cargo directory source and pins the legacy provider source records from an externally identified source bundle without consulting ambient caches or the network.

#### Scenario: Verified bundle hydrates a fresh clone

GIVEN a fresh Mantle clone has no `vendor-deps/`, an operator supplies a bootstrap source bundle, and an independently obtained expected manifest BLAKE3 matches that bundle
WHEN Mantle hydrates the self-build inputs
THEN it MUST validate exactly one vendored-Cargo record plus the required provider archive and provider manifest records
AND it MUST validate the materialized Cargo directory against the clone's `Cargo.lock`, `.cargo/vendor-config.toml`, package checksums, and file checksums before publication
AND it MUST publish `vendor-deps/` atomically without replacement, import and pin the provider records, and bind the successful report to the manifest and vendor content identities.

#### Scenario: Hydrated provider is available without network

GIVEN fresh Mantle source state was hydrated from the verified bundle
WHEN `bootstrap --fetch --offline-source-preflight` selects the same legacy provider URL with network access unavailable
THEN Mantle MUST materialize the provider from pinned source state
AND it MUST NOT silently downgrade to a live network fetch.

#### Scenario: Invalid hydration fails without clobbering outputs

GIVEN the expected manifest BLAKE3 is wrong, the bundle is tampered, a required record is missing or duplicated, the vendored Cargo payload fails lock/checksum validation, source-state persistence fails, or `vendor-deps/` already exists
WHEN hydration runs
THEN Mantle MUST fail with a deterministic diagnostic
AND it MUST NOT replace an existing path, leave a partially published vendor directory, emit a success report, or mark invalid provider state ready.

#### Scenario: Hydration evidence remains bounded

GIVEN fresh-clone hydration and offline provider preflight succeed
WHEN documentation, lifecycle evidence, or an operator report cites the result
THEN the claim MUST be limited to the declared Cargo and provider source/input payloads being locally available and identity-matched
AND it MUST NOT claim fixed-point self-build success, completeness for undeclared future bootstrap sources, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

### Requirement: Hydrated fresh clones prove fixed-point self-hosting without live source acquisition

r[bootstrap_inventory.hydrated_fresh_clone_fixed_point] Mantle MUST provide a bounded proof workflow in which an externally identified source bundle hydrates a fresh clone with the complete evaluated fixed-fetch source closure and both self-build stages fail closed before any undeclared live source acquisition.

#### Scenario: Connected producer exports the complete proof source closure

GIVEN committed Mantle self-build roots contain fixed-output URL or Git sources
WHEN a connected producer creates the full-proof source bundle
THEN Mantle MUST evaluate the selected proof roots, materialize every non-local fixed-fetch payload through the ordinary fixed-output verifier, and bind those payloads to their URL, revision when applicable, hash mode, expected hash, and content BLAKE3 identities
AND the bundle MUST also carry the exact fresh-clone vendored Cargo, provider archive, and provider manifest records without treating URLs and hashes alone as payload availability.

#### Scenario: Fresh proof stages forbid unmatched source fetches

GIVEN a fresh clone hydrates an independently identity-matched full-proof bundle into fresh source state
WHEN stage0 or stage2 requests a builtin fixed-output source
THEN the request MUST resolve from a matching pinned source record or fail before URL, Git, proxy, DNS, or other live acquisition
AND successful stage evidence MUST bind the enforced source-state identity and report zero live-fetch events.

#### Scenario: Hydrated fresh clone reaches the admitted fixed point

GIVEN the clone initially lacks `vendor-deps/`, Cargo source payloads, imported source state, and prior proof outputs
WHEN the full stage0 → stage1 → stage2 proof runs with the authenticated source bundle and live source acquisition forbidden
THEN locked Cargo metadata MUST succeed from the hydrated vendor tree, stage1 and stage2 MUST complete from fresh stage state using the same authenticated source authority, and their binary BLAKE3 digests MUST match
AND the proof bundle MUST retain contracted hydration, source-policy, stage, fallback, tool, and fixed-point evidence without depending on the producer checkout's local paths.

#### Scenario: Incomplete or invalid source authority fails closed

GIVEN the expected bundle identity is wrong, a required record is missing, duplicated, stale, unpinned, tampered, unsupported, or mapped to the wrong URL or revision
WHEN hydration, preflight, or either proof stage runs
THEN Mantle MUST fail with a deterministic diagnostic before emitting admitted fixed-point success
AND it MUST NOT silently fetch the missing source, publish partial hydration success, reuse arbitrary producer build outputs, or update a successful-proof alias.

#### Scenario: Fresh-clone fixed-point evidence remains bounded

GIVEN the hydrated fresh-clone proof succeeds
WHEN operators or release tooling cite the result
THEN the claim MUST identify the committed source closure, expected bundle BLAKE3, source-state BLAKE3, provider kind, platform, proof mode, stage binary digests, and live-fetch denial result
AND it MUST NOT claim full-source bootstrap, compiler correctness, seed trust removal, bit-for-bit release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

### Requirement: Source-built bootstrap seed provider promotion

r[bootstrap_inventory.source_built_seed_provider] Mantle MUST select its source-built bootstrap seed provider only after the complete declared compiler, libc, binutils, and normalized-provider chain has produced real runtime-validated artifacts without legacy-provider, pass1-delegation, generated-stub, or undeclared host-tool fallback.

#### Scenario: source-built chain produces the normalized provider

GIVEN the declared bootstrap source closure starts from the audited bootstrap seed and contains the TinyCC, musl, GCC, binutils, and required build-tool sources
WHEN Mantle builds the source-provider candidate
THEN every compiler ladder stage MUST compile its admitted artifacts from declared source inputs rather than substitute empty objects, generated stubs, or wrappers that delegate compiler behavior to a predecessor
AND the normalized output MUST contain executable target-prefixed GCC/G++, assembler, linker, archive, inspection, and strip tools plus headers, CRT objects, `libc.so`, `libc.a`, `libgcc_s.so.1`, and provider metadata bound to the source and output identities.

#### Scenario: provider admission executes positive tool and runtime smoke

GIVEN a candidate normalized provider has been built
WHEN Mantle evaluates it for selection
THEN admission MUST execute bounded C, C++, assembly/link, archive, object-inspection, static-libc, dynamic-libc, and libgcc runtime smoke against the candidate's own compiler internals and sysroot
AND executable-bit checks, `--version` output, contract text, or metadata shape alone MUST NOT satisfy admission.

#### Scenario: incomplete or bridged provider fails closed

GIVEN any reachable candidate stage contains the legacy musl.cc provider, `seed-legacy.ncl`, TinyCC-delegating GCC wrappers, fabricated generator/backend objects, missing compiler internals, missing runtime surfaces, undeclared host tools, or stale/tampered evidence
WHEN construction, admission, selection, or self-build runs
THEN Mantle MUST fail with a deterministic diagnostic before changing the selected seed or emitting source-built-provider success
AND it MUST retain the honest legacy development path or blocked status without relabeling host-assisted source-root materialization as full-source proof.

#### Scenario: selected provider drives authenticated fixed-point self-build

GIVEN the source-built provider passes runtime admission from committed implementation source and its complete fixed-fetch closure is independently authenticated
WHEN a fresh clone runs stage0 → stage1 → stage2 with undeclared live source acquisition forbidden
THEN both stages MUST bind the same source authority and admitted provider identity, report zero undeclared live fetches, and produce matching stage1/stage2 Mantle binary BLAKE3 digests
AND the proof bundle MUST retain provider construction, admission, source-policy, tool/runtime smoke, fallback, and fixed-point evidence.

#### Scenario: source-built-provider claims remain bounded

GIVEN provider construction and fixed-point self-build succeed
WHEN operators or release tooling cite the result
THEN the claim MUST identify the bootstrap seed, source closure, orchestration boundary, platform, provider/output identities, tool/runtime smoke, and fixed-point digests
AND it MUST NOT claim compiler correctness, bootstrap-seed correctness, independent rebuild agreement, bit-for-bit release reproducibility, deployment success, or full Cargo compatibility without separate evidence.

### Requirement: Perl 5.005_03 GCC generator runtime is causally validated

r[bootstrap_inventory.perl_5_005_03_gcc_runtime] Mantle MUST promote `bootstrap/perl-5.005_03-gcc.ncl` as a source-built generator only when a bounded comparison identifies the smallest required correction in the target or its declared predecessor closure and the resulting interpreter passes positive execution and negative parser behavior under those declared bootstrap inputs.

#### Scenario: precursor failure is not Perl evidence

GIVEN a canonical or diagnostic build fails in stage0, source acquisition, provider construction, store materialization, or another predecessor before the Perl builder runs
WHEN Mantle records the comparison outcome
THEN the outcome MUST identify the precursor and exact failure class
AND it MUST NOT classify the run as evidence for or against Perl optimization, ABI configuration, compiler-generation behavior, or runtime success.

#### Scenario: one-variable candidates preserve causal isolation

GIVEN optimization mode, LP64 size configuration, and compiler generation are plausible independent causes
WHEN Mantle compares diagnostic candidates
THEN each first-round candidate MUST vary only one mechanism while preserving the same Perl source, generated-header path, libc, binutils, bounded source list, and smoke contract
AND the evidence MUST record the command, derivation identity, terminal status, saved log, and first discriminating failure or success.

#### Scenario: canonical target passes after predecessor repair

GIVEN a target build first fails before Perl 5.005_03 because a declared predecessor violates its normalized direct-input contract
WHEN Mantle repairs that predecessor and the unchanged target compiler/configuration passes the full behavioral contract
THEN the comparison MUST identify the predecessor repair as the smallest causal correction
AND it MUST reject unexecuted target-local variants as unnecessary rather than imply they were proven fixes.

#### Scenario: viable generator passes positive and negative behavior

GIVEN a Perl 5.005_03 candidate builds and installs an interpreter
WHEN Mantle evaluates it for canonical promotion
THEN the interpreter MUST report version 5.005_03, execute the declared arithmetic program with the expected result, reject malformed Perl source with nonzero status and empty stdout, and have the declared ELF64 shape
AND compilation, installation, version output, or ELF inspection alone MUST NOT satisfy promotion.

#### Scenario: canonical correction is minimal and temporary variants are removed

GIVEN the target or one of its declared predecessors requires correction to satisfy the full behavioral contract
WHEN Mantle updates the bootstrap chain
THEN it MUST select the smallest evidence-backed semantic correction, preserve named ABI and smoke constants, and add deterministic regression coverage
AND temporary hidden diagnostic derivations MUST be deleted after their dispositions are preserved in lifecycle evidence.

#### Scenario: generator claims remain bounded

GIVEN the corrected Perl 5.005_03 interpreter passes its focused runtime contract
WHEN operators or downstream bootstrap stages cite the result
THEN the claim MUST identify the source, declared predecessor chain, store identity, runtime checks, and evidence location
AND it MUST NOT claim compiler correctness, normalized-provider admission, independent reproducibility, downstream Perl 5.6.2 success, or whole-bootstrap correctness without separate evidence.

### Requirement: Early native bootstrap parity is artifact-backed

r[bootstrap_inventory.early_native_bootstrap_parity] Mantle MUST mark the `binutils.tcc` and `gcc.4.0` bootstrap parity rows complete only from independently validated current source-built artifacts and BLAKE3-bound receipts, never from later-provider success, metadata shape, version output, or compatibility-bridge prose.

#### Scenario: TCC binutils handoff is complete

GIVEN authenticated TCC-era sources and predecessor tools
WHEN Mantle builds and evaluates the `binutils.tcc` row
THEN the declared assembler, linker, archive, ranlib, nm, objcopy, object-format, and relocation surfaces MUST be real source-built outputs with positive and malformed-input behavior evidence
AND omitted tools, predecessor delegation, host fallback, incomplete archive members, or stale receipt identities MUST keep the row blocked.

#### Scenario: GCC 4.0 native handoff is complete

GIVEN the admitted early binutils handoff and authenticated regenerated GCC 4.0 sources
WHEN Mantle builds and evaluates the `gcc.4.0` row
THEN the C and C++ drivers, compiler internals, required generators, demangler, libgcc and exception/runtime artifacts MUST be built from declared sources and pass the receipt-defined positive and rejection matrix
AND TinyCC delegation, fabricated objects, release-generated substitution, missing compiler internals, or use of the configure preprocessing bridge beyond its audited authority MUST keep the row blocked.

#### Scenario: row evidence is not interchangeable

GIVEN one early row has complete evidence and the other is absent, stale, malformed, or incomplete
WHEN the bootstrap parity report is generated
THEN Mantle MUST promote only the independently complete row
AND axis status MUST retain the incomplete row as a blocker with a deterministic reason.

#### Scenario: early parity claims remain bounded

GIVEN both early rows are complete
WHEN the evidence is cited
THEN the claim MUST identify sources, predecessors, generated artifacts, outputs, behavior tests, rejection tests, and fallback results
AND it MUST NOT claim general compiler, assembler, linker, seed, or bootstrap correctness.

### Requirement: Final native toolchain parity preserves derivational lineage

r[bootstrap_inventory.final_native_toolchain_parity] Mantle MUST mark `gcc.4.7`, `gcc.10`, and `full-musl-binutils` complete only when current artifacts form a BLAKE3-bound derivational closure from the admitted early compiler boundary through regenerated compiler stages to final libc and binutils outputs without state-pinned overlay, impure transitive, host-tool, or undeclared generated-source substitution.

#### Scenario: GCC 4.7 follows the admitted predecessor

GIVEN the early native GCC boundary is complete
WHEN Mantle generates, builds, and evaluates GCC 4.7
THEN required generated sources and compiler/runtime artifacts MUST be produced by declared source-built generators and the admitted immediate predecessor
AND release-generated substitution, wrong-predecessor output, host compilation, state-pinned overlay input, or incomplete receipts MUST keep `gcc.4.7` blocked.

#### Scenario: GCC 10 follows GCC 4.7

GIVEN GCC 4.7 has complete stage-local evidence
WHEN Mantle generates, builds, and evaluates GCC 10
THEN GCC 10 C/C++ drivers, compiler internals, libgcc, C++ runtime, and required generated artifacts MUST bind the GCC 4.7 receipt and authenticated source records
AND simple compile success MUST NOT hide undeclared host tools, missing runtime members, stale generated files, or predecessor substitution.

#### Scenario: final musl and binutils close the provider

GIVEN GCC 10 has complete stage-local evidence
WHEN Mantle builds final musl and binutils and normalizes the provider
THEN the output MUST bind headers, CRT, static/shared libc, libgcc and C++ runtimes, dynamic interpreter, assembler, linker, archive, ranlib, inspection, strip, relocation, and copied-tree behavior to current artifacts
AND missing surfaces, embedded unavailable paths, malformed-input crashes/timeouts, impure closure members, or stale receipts MUST keep `full-musl-binutils` blocked.

#### Scenario: final parity claims remain bounded

GIVEN all three final rows are complete
WHEN the result is reported
THEN Mantle MUST identify every immediate predecessor, source/generator receipt, output digest, runtime/rejection result, closure scan, and relocation result
AND it MUST NOT claim compiler, libc, binutils, seed, or whole-bootstrap correctness.
