## ADDED Requirements

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
