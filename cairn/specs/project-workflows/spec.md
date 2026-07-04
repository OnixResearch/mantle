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

### Requirement: Project inputs support forge-agnostic VCS kinds [r[project_workflows.forge_agnostic_vcs_inputs]]

Mantle MUST support project input kinds for Darcs, Pijul, and Fossil without privileged forge-specific URL schemes. Each VCS kind MUST use explicit repository or remote URLs, VCS-native selectors, mirrors when verifiable, locked VCS identity metadata, and a locked source-tree content digest.

#### Scenario: Darcs input locks native identity [r[project_workflows.forge_agnostic_vcs_inputs.scenario.darcs]]

- GIVEN a project input declares a Darcs repository with a tag or context selector
- WHEN Mantle refreshes or locks the input
- THEN Mantle MUST record Darcs-native identity metadata such as context or weak-hash facts when available
- AND it MUST record a source-tree content digest for the materialized checkout.

#### Scenario: Pijul input locks channel state [r[project_workflows.forge_agnostic_vcs_inputs.scenario.pijul]]

- GIVEN a project input declares a Pijul remote with channel, state, or change selection
- WHEN Mantle refreshes or locks the input
- THEN Mantle MUST record the selected Pijul identity facts
- AND it MUST verify mirrors or refreshed material against the same locked source-tree content digest.

#### Scenario: Fossil input locks check-in identity [r[project_workflows.forge_agnostic_vcs_inputs.scenario.fossil]]

- GIVEN a project input declares a Fossil repository with branch, tag, or check-in selection
- WHEN Mantle refreshes or locks the input
- THEN Mantle MUST record the selected Fossil identity facts
- AND it MUST produce generated input data that refers to the locked source material without requiring forge-specific semantics.

#### Scenario: Forge shortcuts are not source semantics [r[project_workflows.forge_agnostic_vcs_inputs.scenario.no-forge-shortcut]]

- GIVEN a repository is hosted on GitHub, GitLab, Codeberg, a Darcs hub, a Pijul nest, or a Fossil host
- WHEN Mantle validates the project input
- THEN Mantle MUST treat the host as a URL endpoint only
- AND it MUST NOT require or privilege host-specific shorthand schemes as part of source identity.

### Requirement: VCS inputs fail closed when identity cannot be proven [r[project_workflows.vcs_input_fail_closed]]

Mantle MUST fail closed when a non-Git VCS input cannot prove the selected source identity, content digest, or mirror equivalence. Missing tools or unsupported VCS subfeatures MUST be reported as deterministic blockers rather than successful source support.

#### Scenario: Missing VCS tool is an unsupported blocker [r[project_workflows.vcs_input_fail_closed.scenario.missing-tool]]

- GIVEN a project input requires a VCS adapter whose implementation or external tool is unavailable
- WHEN Mantle plans, refreshes, or builds the input
- THEN Mantle MUST emit an unsupported-VCS-tool diagnostic
- AND it MUST NOT update the lockfile or report the input as fetched.

#### Scenario: Mirror mismatch is rejected [r[project_workflows.vcs_input_fail_closed.scenario.mirror-mismatch]]

- GIVEN a VCS input mirror resolves to material that does not match the locked VCS identity or source-tree digest
- WHEN Mantle verifies the mirror result
- THEN Mantle MUST reject that mirror result
- AND it MUST continue to another mirror only if the next result can prove the same identity.

### Requirement: Project shell profiles are named build-tool handoffs [r[project_workflows.named_shell_profiles]]

Mantle MUST support named project shell profiles that resolve to explicit build inputs, environment entries, path entries, and activation sidecar data. Shell profiles MUST remain build-tool handoffs and MUST NOT cause Mantle core to interpret Nix flakes, Onix modules, service lifecycle semantics, or frontend-specific package sets.

#### Scenario: Default shell profile resolves deterministically [r[project_workflows.named_shell_profiles.scenario.default]]

- GIVEN a project declares `build`, `dev`, and optionally `default` shell profiles
- WHEN an operator runs `mantle shell` without an explicit profile name
- THEN Mantle MUST resolve the default profile deterministically
- AND diagnostics MUST identify which named profile was selected.

#### Scenario: Explicit profile lowers to activation plan [r[project_workflows.named_shell_profiles.scenario.explicit]]

- GIVEN a project declares a named shell profile with explicit build inputs, environment entries, and path entries
- WHEN an operator runs `mantle shell <name>` or an equivalent non-interactive shell command
- THEN Mantle MUST lower that profile to ordinary Mantle build inputs and activation sidecar data
- AND the pure shell-planning core MUST receive owned normalized data rather than reading the filesystem, environment, or frontend state.

#### Scenario: Invalid profile fails before activation [r[project_workflows.named_shell_profiles.scenario.invalid]]

- GIVEN a profile name is invalid, the default is ambiguous, env entries collide after normalization, path entries are unsupported, or adapter paths cannot be represented safely
- WHEN Mantle validates or activates the shell profile
- THEN Mantle MUST fail with deterministic diagnostics before executing a shell or hook
- AND it MUST NOT silently fall back to another profile.

#### Scenario: Services remain outside shell profile semantics [r[project_workflows.named_shell_profiles.scenario.no-services]]

- GIVEN a project or frontend wants long-running development services such as databases, queues, or daemons
- WHEN Mantle processes named shell profiles
- THEN Mantle MUST reject service lifecycle declarations or treat service descriptors only as opaque generated data under a separate explicit contract
- AND it MUST NOT start, stop, supervise, restart, or health-check services as part of shell profile activation.

#### Scenario: Shell profile claim is bounded [r[project_workflows.named_shell_profiles.scenario.non-claim]]

- GIVEN a named shell profile activates successfully
- WHEN Mantle renders human output, JSON output, docs, or evidence
- THEN the claim MAY state that the selected shell activation plan was produced and applied under recorded inputs
- AND it MUST NOT claim build success, test success, service readiness, deployability, or release reproducibility without separate evidence.

### Requirement: Dev shell activation is decoupled from build identity [r[project_workflows.dev_shell_decoupling]]

Mantle MUST keep dev shell activation separate from package build action identity, file generation, lock refresh, release evidence, and reproducibility claims. A dev shell MAY reuse Mantle-built packages, but entering or planning the shell MUST NOT change build hashes, generated files, lockfiles, project manifests, or proof state unless an operator invokes a separate explicit mutating command.

#### Scenario: Dev shell does not affect action identity [r[project_workflows.dev_shell_decoupling.scenario.action-identity]]

- GIVEN a project has package build declarations and a `dev` shell profile with extra convenience tools or environment entries
- WHEN Mantle computes package action specs before and after planning or activating the dev shell
- THEN the package action identity MUST remain determined by the package declarations and declared build inputs
- AND `shells.dev` MUST NOT become an implicit input to package build hashes.

#### Scenario: Shell activation is non-mutating by default [r[project_workflows.dev_shell_decoupling.scenario.non-mutating]]

- GIVEN a project has generated files, lock entries, refreshable inputs, and named shell profiles
- WHEN an operator runs `mantle shell` or `mantle shell dev`
- THEN Mantle MUST NOT regenerate files, rewrite lockfiles, refresh inputs, edit project manifests, or mutate release/proof evidence as part of shell activation
- AND any required mutation MUST be routed through a separate explicit command such as file generation, refresh, or build.

#### Scenario: Shell activation receipt is separate evidence [r[project_workflows.dev_shell_decoupling.scenario.separate-receipt]]

- GIVEN Mantle records evidence for shell activation
- WHEN human output, JSON output, release evidence, or task evidence cites that shell evidence
- THEN the evidence MUST be a separate shell-activation receipt or equivalent bounded record naming the selected profile and activation plan
- AND build reports, release evidence, and reproducibility claims MUST NOT treat that receipt as build or release proof unless a separate requirement explicitly admits the narrower claim.

#### Scenario: Dev shell overclaiming is rejected [r[project_workflows.dev_shell_decoupling.scenario.non-claim]]

- GIVEN a dev shell activates successfully
- WHEN Mantle renders diagnostics, docs, release notes, task evidence, or final status
- THEN the claim MAY state only that the selected dev shell activation plan was applied or made available
- AND it MUST NOT claim package build success, test success, file generation freshness, lock freshness, service readiness, release reproducibility, or build action correctness without separate current evidence.

### Requirement: Project file generation is explicit and reviewable [r[project_workflows.project_filegen]]

Mantle MUST support project-declared generated files through an explicit no-mutate planning workflow and an explicit apply workflow. File generation declarations MUST name target paths, content sources, materialization methods, and content identity, and Mantle MUST reject path escapes, unsupported methods, conflicts, and plan drift before mutation.

#### Scenario: Filegen plan is side-effect free [r[project_workflows.project_filegen.scenario.plan]]

- GIVEN a project declares generated files with target paths, content sources, and materialization methods
- WHEN an operator runs the file generation plan command
- THEN Mantle MUST report deterministic create, update, unchanged, stale, and conflict operations without writing files, lockfiles, generated inputs, or store state
- AND the plan MUST include enough content identity for review before apply.

#### Scenario: Filegen apply writes only accepted operations [r[project_workflows.project_filegen.scenario.apply]]

- GIVEN a generated-file plan has no blocking conflicts and the operator explicitly applies it
- WHEN Mantle materializes generated files
- THEN Mantle MUST write only the files and materialization methods named by the verified plan
- AND it MUST fail before writing if current-file facts differ from the reviewed plan.

#### Scenario: Target escapes and conflicts fail closed [r[project_workflows.project_filegen.scenario.conflict]]

- GIVEN a generated file target is absolute, normalizes above the project root, collides with an unmanaged file, or uses an unsupported materialization method
- WHEN Mantle plans or applies file generation
- THEN Mantle MUST emit a deterministic blocker naming the unsafe target or method
- AND it MUST NOT create, replace, symlink, or delete that target.

### Requirement: Generated project files may be typed content [r[project_workflows.project_filegen_typed_content]]

Mantle MUST allow generated file declarations to bind content to a Nickel contract or schema-derived contract when the project supplies one. Contract validation MUST happen before file materialization, and success MUST be reported as generated-content validation only, not as frontend deployability or build success.

#### Scenario: Valid generated content is materializable [r[project_workflows.project_filegen_typed_content.scenario.valid]]

- GIVEN a generated file declaration includes content plus a contract binding
- WHEN Mantle validates the generated content before apply
- THEN Mantle MUST accept the content only if it satisfies the declared contract
- AND the filegen evidence MUST bind the contract identity and generated content digest.

#### Scenario: Invalid generated content blocks apply [r[project_workflows.project_filegen_typed_content.scenario.invalid]]

- GIVEN generated content fails its declared contract or schema-derived contract
- WHEN Mantle plans or applies file generation
- THEN Mantle MUST report a deterministic contract-validation diagnostic
- AND it MUST NOT materialize the invalid generated file.

#### Scenario: Typed generated file claim is bounded [r[project_workflows.project_filegen_typed_content.scenario.non-claim]]

- GIVEN generated content satisfies its declared contract and is materialized
- WHEN Mantle renders human output, JSON output, docs, or evidence
- THEN the claim MAY state that the generated file matches the declared content digest and contract identity
- AND it MUST NOT claim deployability, service readiness, frontend module correctness, or build success without separate evidence.

### Requirement: Cargo import accepts declared vendored source material

r[project_workflows.cargo_import_vendored_sources] Mantle MUST allow `mantle import cargo` to scaffold the offline Cargo project-build lane for workspaces with registry or git dependencies only when the required dependency material is present as an explicit vendored source input. The import planner MUST bind accepted vendored material to `Cargo.lock`, Cargo checksum metadata, source replacement configuration, Mantle BLAKE3 identities, and generated `vendor_src` / `vendor_name` inputs, and MUST fail closed instead of consulting ambient Cargo caches or enabling network access.

#### Scenario: vendored registry material is accepted

GIVEN a Cargo workspace has a `Cargo.lock` entry for a registry dependency
AND source replacement configuration points to a local vendored directory containing that package
AND the vendored package files match Cargo checksum metadata
WHEN an operator runs `mantle import cargo --plan` or `--apply`
THEN Mantle MAY report the dependency material as an accepted vendored source input
AND generated project files MUST pass that source input explicitly to `mantle.offlineCargoPackage`.

#### Scenario: undeclared dependency material blocks import

GIVEN a Cargo workspace has registry or git dependencies without accepted vendored source material
WHEN an operator runs `mantle import cargo --plan` or `--apply`
THEN Mantle MUST emit deterministic blockers naming the missing or unsupported dependency material
AND it MUST NOT generate partial files that imply the package is ready for `mantle build`.

#### Scenario: stale vendor checksums fail closed

GIVEN a vendored package directory exists but its files, lockfile identity, package version, source identity, or `.cargo-checksum.json` metadata do not match the selected dependency facts
WHEN Mantle validates the import plan
THEN Mantle MUST reject the vendored material before applying generated files
AND diagnostics MUST distinguish stale checksum, ambiguous package identity, missing lock entry, and unsupported source replacement classes.

#### Scenario: import does not run vendoring

GIVEN a workspace could be made buildable by running `cargo vendor`, fetching registry material, cloning git dependencies, or reading a user Cargo cache
WHEN Mantle plans Cargo import
THEN Mantle MUST NOT perform those actions
AND the plan MUST report that pre-existing declared source material is required.
