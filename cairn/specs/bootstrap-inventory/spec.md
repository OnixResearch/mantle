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
