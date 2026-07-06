## ADDED Requirements

### Requirement: Lock importer offline proof rail emits bounded versioned evidence

r[project_workflows.lock_importer_proof_rail] Mantle MUST provide a bounded local offline proof rail that exercises the external pin import `--plan` and `--apply` workflow for a supported importer (Nixtamal), where `--plan` is side-effect free and reports planned file operations, mapped inputs/patches, unsupported semantics, and blockers, `--apply` writes only Mantle-owned files named by the plan and fails before writing on conflicts, unsupported semantics, or plan drift, and the rail emits a versioned, redacted, non-overclaiming evidence record stating the generated project remains a build-tool handoff (not Onix/NixOS module semantics).

#### Scenario: plan is side-effect free and reviewable

GIVEN a project contains external Nixtamal pinning files from a supported importer
WHEN an operator runs the import `--plan` command through the rail
THEN Mantle MUST render deterministic planned file operations, mapped inputs, mapped patches, unsupported semantics, and blockers
AND it MUST NOT write project files, lockfiles, generated inputs, store state, or source state.

#### Scenario: apply writes only planned Mantle files and fails on drift

GIVEN an import plan has no blockers and the operator explicitly applies it through the rail
WHEN Mantle writes imported project state
THEN Mantle MUST write only the Mantle-owned files named by the plan
AND it MUST fail before writing if existing files, conflicts, unsupported semantics, or plan drift make the apply unsafe.

#### Scenario: composition semantics are blockers not hidden core semantics

GIVEN an external pinning format includes recursive graph semantics, flake output composition, module-layer behavior, follows-like rewriting, or overlays that are not source pin facts
WHEN Mantle plans import through the rail
THEN Mantle MUST either map those facts into explicit source inputs with bounded meaning or report deterministic blockers
AND it MUST NOT import them as hidden Mantle core semantics.

#### Scenario: evidence is versioned redacted and non-overclaiming

GIVEN the rail emits its evidence record
WHEN the record is rendered
THEN it MUST carry a stable schema version, importer kind, planned operations, mapped inputs/patches, blockers, the no-mutate and only-planned-files assertions, and non-claims
AND it MUST omit raw environment values and unbounded logs and MUST NOT claim build success, deployability, or frontend module correctness from import alone.
