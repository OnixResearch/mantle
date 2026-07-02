# Project Workflows Specification

## Purpose

Defines the `project-workflows` capability.

## Requirements

### Requirement: Offline Cargo builds for Mantle projects [r[project_workflows.offline_cargo_builds]]

Mantle MUST provide a bounded project workflow for building Rust packages by running Cargo inside a Mantle sandbox with explicit offline source-closure material, and MUST label the result as Cargo-orchestrated sandbox evidence rather than Cargo-free native execution.

#### Scenario: Supported Rust project builds offline [r[project_workflows.offline_cargo_builds.scenario.success]]

- GIVEN a Mantle project declares a Rust package with package source, `Cargo.lock` identity, vendored dependency source material, selected target/profile, toolchain inputs, and runnable output contract
- WHEN `mantle build .#name` or `mantle run .#name` builds that package
- THEN Mantle MUST lower the package to ordinary build actions that run Cargo with explicit offline inputs inside the sandbox
- AND the build report MUST bind the source closure, selected toolchain, output path, artifact attestation, and bounded Cargo-inside-sandbox claim.

#### Scenario: Missing source material fails closed [r[project_workflows.offline_cargo_builds.scenario.missing-source]]

- GIVEN a declared Rust package would require undeclared registry cache, git checkout, target-directory state, network access, or missing vendored source material
- WHEN Mantle plans or builds the package
- THEN Mantle MUST fail with a deterministic source-closure or offline-build blocker before accepting the output
- AND it MUST NOT search ambient Cargo caches or silently enable network access to repair the missing material.

#### Scenario: Claims remain bounded [r[project_workflows.offline_cargo_builds.scenario.non-claims]]

- GIVEN an offline Cargo package build succeeds through Mantle
- WHEN Mantle renders human output, JSON reports, attestations, docs, or task evidence
- THEN the evidence MAY claim that the declared Cargo action produced the inspected outputs under the recorded Mantle sandbox policy
- AND it MUST NOT claim Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness unless separate evidence exists.

### Requirement: Cargo project import scaffold [r[project_workflows.cargo_import_scaffold]]

Mantle MUST provide a no-mutate planning workflow and an explicit apply workflow that scaffold Mantle project files from supported Cargo workspace facts, and MUST keep the generated surface build-shaped rather than module-layer-shaped.

#### Scenario: Import plan is reviewable and side-effect free [r[project_workflows.cargo_import_scaffold.scenario.plan]]

- GIVEN a Cargo workspace has supported package, target, lockfile, and source-closure facts
- WHEN the operator requests an import plan
- THEN Mantle MUST report deterministic file operations, selected packages, default target choice, source inputs, content digests, and any blockers without mutating files or store state
- AND the plan MUST be sufficient for review before apply.

#### Scenario: Apply writes only accepted project files [r[project_workflows.cargo_import_scaffold.scenario.apply]]

- GIVEN an import plan has no blocking conflicts
- WHEN the operator explicitly applies that plan
- THEN Mantle MUST write only the bounded Mantle-owned project files named by the plan
- AND it MUST fail before writing if existing files, mixed legacy/canonical surfaces, unsupported source material, or ambiguous package selection make the plan unsafe.

#### Scenario: Unsupported Cargo surfaces block scaffold claims [r[project_workflows.cargo_import_scaffold.scenario.unsupported]]

- GIVEN a Cargo workspace requires behavior outside the supported import surface, such as undeclared registry material, unsupported target kinds, missing lockfile facts, or ambiguous binary selection
- WHEN Mantle plans the import
- THEN Mantle MUST emit deterministic blockers naming the unsupported surface
- AND it MUST NOT generate partial files that imply the workspace is ready for `mantle build`.

#### Scenario: Generated project remains a build-tool handoff [r[project_workflows.cargo_import_scaffold.scenario.boundary]]

- GIVEN generated project files are produced from Cargo workspace facts
- WHEN those files are evaluated by Mantle
- THEN they MUST describe concrete derivations, package outputs, source inputs, checks, or opaque build data
- AND they MUST NOT introduce Onix/NixOS-style module semantics into Mantle core.

### Requirement: Mantle checks project workflow soundness [r[project_workflows.project_soundness_checks]]

Mantle MUST check static soundness across the project manifest, lockfile, generated inputs, patches, mirrors, hash algorithms, freshness metadata, trust policy, fetch policy, retention roots, and orphaned lock entries. Default soundness checks MUST be no-network unless an explicit mode documents freshness or trust execution.

#### Scenario: Static soundness checks clean project state [r[project_workflows.project_soundness_checks.scenario.clean]]

- GIVEN a project manifest, lockfile, generated inputs, patch definitions, mirrors, hash algorithms, fetch policy, trust policy, and retention roots agree
- WHEN `mantle check` runs in default mode
- THEN Mantle MUST report the static project state as sound
- AND it MUST NOT contact the network, run freshness commands, or fetch missing source material.

#### Scenario: Manifest lock mismatch is classified [r[project_workflows.project_soundness_checks.scenario.kind-mismatch]]

- GIVEN a manifest input and lockfile entry disagree on input kind, source identity, hash algorithm, patch set, fetch policy, or trust policy shape
- WHEN `mantle check` compares project state
- THEN Mantle MUST report a deterministic mismatch diagnostic naming the input and mismatched class
- AND it MUST NOT silently merge or refresh the entry.

#### Scenario: Generated inputs must match lockfile [r[project_workflows.project_soundness_checks.scenario.generated-stale]]

- GIVEN `.mantle/inputs.ncl` or equivalent generated input state is stale, missing, or inconsistent with the lockfile
- WHEN `mantle check` runs
- THEN Mantle MUST report a generated-input-stale diagnostic
- AND it MUST NOT rewrite generated inputs unless an explicit repair or refresh command is selected.

#### Scenario: Optional probe mode labels network behavior [r[project_workflows.project_soundness_checks.scenario.probe-mode]]

- GIVEN an operator explicitly requests soundness checks that execute freshness probes or trust verification
- WHEN those checks run
- THEN Mantle MUST label the mode and possible network or process behavior in diagnostics or JSON metadata
- AND failed probes or trust checks MUST be reported as soundness issues without rewriting lock entries.

### Requirement: Project inputs declare bounded freshness probes [r[project_workflows.freshness_probes]]

Mantle MUST support versioned, bounded freshness probes for project inputs and patches. Probe definitions MUST be manifest data, and probe execution MUST stay in the imperative shell while the functional core receives normalized observation records. Supported probe families MUST include built-in Git reference observations, HTTP text or JSON observations, local file or directory observations, and explicitly bounded command observations.

#### Scenario: Built-in probe produces normalized observation [r[project_workflows.freshness_probes.scenario.builtin]]

- GIVEN a project input declares a supported built-in freshness probe
- WHEN Mantle evaluates freshness for that input
- THEN the shell MUST produce a normalized observation containing the input name, probe kind, observed value, value digest, status, and bounded diagnostics
- AND the pure core MUST classify the observation without reading the network, filesystem, environment, or process state.

#### Scenario: Command probe is bounded [r[project_workflows.freshness_probes.scenario.command-bounded]]

- GIVEN a project input declares a command freshness probe
- WHEN Mantle runs the probe
- THEN Mantle MUST enforce explicit argv, cwd, environment, timeout, output-size, and success-status limits
- AND empty output, oversized output, timeout, missing executable, invalid UTF-8 where UTF-8 is required, or non-success status MUST become deterministic probe failures.

#### Scenario: Offline mode does not run network probes [r[project_workflows.freshness_probes.scenario.offline]]

- GIVEN a freshness probe requires network access
- WHEN Mantle runs list-stale, refresh planning, check, or offline preflight in a no-network mode
- THEN Mantle MUST report the probe as network-required or unavailable
- AND it MUST NOT contact the network or silently treat the old lock value as freshly observed.

### Requirement: Refresh uses freshness observations without overclaiming [r[project_workflows.freshness_probe_refresh]]

Mantle MUST use normalized freshness observations to drive `mantle list-stale` and `mantle refresh` decisions. Freshness MUST determine whether a locked input should be considered stale, but it MUST NOT by itself prove source integrity, trust, build success, or reproducibility.

#### Scenario: List stale does not mutate [r[project_workflows.freshness_probe_refresh.scenario.list-stale]]

- GIVEN project inputs have existing lockfile freshness values
- WHEN `mantle list-stale` compares current observations against the lockfile
- THEN Mantle MUST report stale, unchanged, failed, skipped, or network-required inputs deterministically
- AND it MUST NOT write the lockfile, generated inputs, store state, or retention roots.

#### Scenario: Refresh updates only selected stale inputs [r[project_workflows.freshness_probe_refresh.scenario.refresh-selected]]

- GIVEN selected project inputs have valid freshness observations and at least one differs from the lockfile
- WHEN `mantle refresh` runs for those inputs
- THEN Mantle MUST fetch and hash only selected stale inputs plus required patches or trust material
- AND unchanged or failed inputs MUST remain unchanged in the lockfile unless an explicit repair mode is selected.

#### Scenario: Freshness value may feed a bounded template [r[project_workflows.freshness_probe_refresh.scenario.template]]

- GIVEN an input URL or reference template uses the validated freshness value
- WHEN Mantle renders the fetch plan
- THEN Mantle MUST render the template through bounded pure logic
- AND undefined variables, invalid rendered syntax, or oversized rendered values MUST fail before fetch or lock update.

### Requirement: Mantle plans and applies external pin imports safely [r[project_workflows.project_lock_importers]]

Mantle MUST provide a no-mutate import planning workflow and an explicit apply workflow for converting supported external pinning files into Mantle project manifests, lockfiles, and generated input files. Import planning MUST report preserved semantics, rewritten semantics, blockers, planned file operations, and bounded non-claims before any mutation.

#### Scenario: Import plan is side-effect free [r[project_workflows.project_lock_importers.scenario.plan]]

- GIVEN a project contains external pinning files from a supported importer
- WHEN an operator runs the import plan command
- THEN Mantle MUST render deterministic planned file operations, mapped inputs, mapped patches, unsupported semantics, and blockers
- AND it MUST NOT write project files, lockfiles, generated inputs, store state, or source state.

#### Scenario: Apply writes only planned Mantle files [r[project_workflows.project_lock_importers.scenario.apply]]

- GIVEN an import plan has no blockers and the operator explicitly applies it
- WHEN Mantle writes imported project state
- THEN Mantle MUST write only Mantle-owned files named by the plan
- AND it MUST fail before writing if existing files, conflicts, unsupported semantics, or plan drift make the apply unsafe.

#### Scenario: Composition semantics are blockers [r[project_workflows.project_lock_importers.scenario.composition-blocker]]

- GIVEN an external pinning format includes recursive graph semantics, flake output composition, module-layer behavior, follows-like rewriting, or overlays that are not source pin facts
- WHEN Mantle plans import
- THEN Mantle MUST either map those facts into explicit source inputs with bounded meaning or report deterministic blockers
- AND it MUST NOT import them as hidden Mantle core semantics.

### Requirement: Nixtamal import maps supported pinning semantics [r[project_workflows.nixtamal_importer]]

Mantle MUST provide a Nixtamal importer that maps supported Nixtamal pinning semantics into Mantle project workflow data without silent downgrades. The importer MUST preserve or block source kind, URL or repository, mirrors, patches, hash algorithm, expected hash, frozen state, freshness behavior, fetch policy, trust policy, and lock identity where Mantle can represent them.

#### Scenario: Supported Nixtamal inputs are mapped [r[project_workflows.nixtamal_importer.scenario.supported]]

- GIVEN a Nixtamal manifest and lockfile contain supported file, archive, Git, mirror, patch, BLAKE3, frozen, and freshness metadata
- WHEN Mantle plans import
- THEN Mantle MUST produce equivalent Mantle project input and lock plans
- AND the plan MUST identify any syntax or policy rewrites needed for review.

#### Scenario: Unsupported Nixtamal semantics block apply [r[project_workflows.nixtamal_importer.scenario.unsupported]]

- GIVEN a Nixtamal input uses a source kind, freshness command, fetch-time behavior, patch source, hash algorithm, or trust policy that Mantle cannot faithfully model
- WHEN Mantle plans import
- THEN Mantle MUST emit an unsupported-import diagnostic
- AND `apply` MUST refuse to write partial files that imply the unsupported input is ready.

### Requirement: Project inputs declare explicit fetch policy [r[project_workflows.input_fetch_policy]]

Mantle MUST let project inputs declare an explicit fetch policy describing when source material may be acquired or required. The policy model MUST distinguish source material needed before generated inputs or dependent evaluation, source material lowered into ordinary build-time fetch actions, and source material that must already be present in imported/offline source state.

#### Scenario: Policy classifies fetch requirements [r[project_workflows.input_fetch_policy.scenario.classify]]

- GIVEN a project manifest declares inputs with different fetch policies
- WHEN Mantle plans refresh, generated inputs, build actions, or source bundle requirements
- THEN Mantle MUST classify each input as generation-material, build-fetch-action, imported-source-required, already-present, unsupported, or conflicting
- AND the classification MUST be deterministic for the same manifest, lockfile, and source-state facts.

#### Scenario: Incompatible policy fails closed [r[project_workflows.input_fetch_policy.scenario.incompatible]]

- GIVEN an input has patches, trust policy, template dependencies, or evaluation-time consumers that cannot be honored by its selected fetch policy
- WHEN Mantle validates the manifest or plans the input
- THEN Mantle MUST reject the incompatible policy with deterministic diagnostics
- AND it MUST NOT silently switch to a broader network or fetch mode.

#### Scenario: Policy remains build-tool data [r[project_workflows.input_fetch_policy.scenario.boundary]]

- GIVEN a frontend or project supplies input fetch policy
- WHEN Mantle validates project workflow data
- THEN Mantle MUST treat the policy as frontend-neutral build/source data
- AND it MUST NOT interpret Onix, NixOS, flake output, or module-layer semantics to decide the policy.

### Requirement: Fetch policy controls offline preflight [r[project_workflows.input_fetch_policy_preflight]]

Mantle MUST apply input fetch policy during offline preflight before sandbox execution or remote dispatch. Offline preflight MUST fail closed when a selected build would require network access or unavailable source state under the declared policy.

#### Scenario: Imported source satisfies offline policy [r[project_workflows.input_fetch_policy_preflight.scenario.imported-ready]]

- GIVEN an input's policy requires imported source state
- AND local source state contains a matching source record with the locked digest and identity
- WHEN Mantle runs offline preflight for a selected root
- THEN Mantle MAY classify that input as ready for an offline build attempt
- AND the report MUST bind the source-state or source-bundle digest used for the decision.

#### Scenario: Network-required policy blocks offline build [r[project_workflows.input_fetch_policy_preflight.scenario.network-blocked]]

- GIVEN an input policy would require generation-time or build-time network fetches that are not satisfied by imported source state
- WHEN Mantle runs offline preflight
- THEN Mantle MUST fail before sandbox execution or remote dispatch
- AND diagnostics MUST identify the input, selected policy, and missing source-state class.

### Requirement: Project inputs may declare retention roots

r[project_workflows.input_retention_roots] Mantle MUST support a project-level default `retention` policy and per-input `retention` overrides for source/input material. Retention policy MUST distinguish `{ mode = "untracked" }`, `{ mode = "current" }`, and `{ mode = "recent-generations", generations = <positive bounded integer> }`; generation limits MUST be named, bounded, and validated before roots are treated as durable. Diagnostics MUST distinguish pinned, unpinned, stale-root, missing-root, and garbage-collection-eligible records.

#### Scenario: Current input is pinned [r[project_workflows.input_retention_roots.scenario.current]]

- GIVEN a project input has retention set to track the current locked source
- WHEN Mantle commits a refresh, import, or generated-input update for that input
- THEN Mantle MUST plan or create a retention root bound to the input name, lock digest, source identity, and content digest
- AND project diagnostics MUST report the input as pinned only after the root exists.

#### Scenario: Generation retention keeps bounded history [r[project_workflows.input_retention_roots.scenario.generations]]

- GIVEN a project default or input override retains recent generations
- WHEN Mantle updates the lockfile across multiple generations
- THEN Mantle MUST retain no more than the configured generation limit for that input
- AND generation selection MUST be based on Mantle-owned lock generation facts rather than filesystem timestamp ordering.

#### Scenario: Untracked input stays GC-eligible [r[project_workflows.input_retention_roots.scenario.untracked]]

- GIVEN a project input has retention disabled or untracked
- WHEN Mantle imports or refreshes that input
- THEN Mantle MUST report the source material as garbage-collection eligible unless another explicit root protects it
- AND it MUST NOT imply durable availability for branch switches or rebases.

### Requirement: Retention updates are atomic with project state

r[project_workflows.input_retention_atomicity] Mantle MUST update project retention roots atomically with the lock/source-state transitions they protect by committing same-directory temporary files into `.mantle/retention.json` and root marker records under `.mantle/retention-roots/`. Interrupted or partial retention updates, including `.mantle/retention.json.tmp`, MUST NOT be reported as durable roots for project readiness.

#### Scenario: Interrupted root update is not durable [r[project_workflows.input_retention_atomicity.scenario.interrupted]]

- GIVEN a refresh or import is interrupted after staging source bytes or root metadata
- WHEN Mantle checks project soundness or offline readiness
- THEN Mantle MUST treat uncommitted retention records as absent or quarantined
- AND it MUST NOT report the corresponding input as pinned.

#### Scenario: Stale root is diagnosed [r[project_workflows.input_retention_atomicity.scenario.stale-root]]

- GIVEN a retention root exists for an older lock digest or mismatched source digest outside the configured generation window
- WHEN Mantle checks project soundness
- THEN Mantle MUST report a stale-root diagnostic
- AND it MUST NOT count that root as satisfying current input retention.

### Requirement: Project inputs and patches may require trust policy [r[project_workflows.input_trust_policy]]

Mantle MUST support explicit trust policy for project inputs and patch definitions. A trust policy MUST define supported verifier kind, signature material references, trusted public key identities or fingerprints, required signer or quorum decisions when supported, and binding to the fetched bytes or digest. Required trust policy MUST be verified before a lockfile refresh writes new source or patch hashes.

#### Scenario: Trusted input refresh is accepted [r[project_workflows.input_trust_policy.scenario.accept]]

- GIVEN a project input declares a trust policy and the fetched bytes have matching content hash and valid trust evidence from the configured signer set
- WHEN Mantle refreshes the input
- THEN Mantle MAY accept the lockfile update
- AND the refresh report MUST identify the verified trust policy without exposing secret key material.

#### Scenario: Missing or invalid trust blocks lock update [r[project_workflows.input_trust_policy.scenario.reject]]

- GIVEN a project input or patch requires trust evidence
- WHEN signature material is missing, malformed, invalid, from an untrusted key, detached from the fetched bytes, or verified by an unsupported verifier
- THEN Mantle MUST reject the refresh before writing new lockfile entries
- AND existing lock entries MUST remain unchanged.

#### Scenario: Hash-only input is not signed evidence [r[project_workflows.input_trust_policy.scenario.hash-only]]

- GIVEN an input has a content hash but no trust policy
- WHEN Mantle reports refresh or project soundness
- THEN Mantle MAY claim content integrity against the recorded hash
- AND it MUST NOT claim signer trust or upstream authenticity for that input.
