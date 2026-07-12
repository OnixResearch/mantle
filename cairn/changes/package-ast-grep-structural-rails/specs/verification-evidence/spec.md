# Verification Evidence Specification

## Purpose

Define Mantle's package and evidence boundaries for pinned ast-grep structural rails.

## Requirements

### Requirement: Pinned ast-grep toolchain
r[mantle.ast_grep_structural_rails.toolchain] Mantle MUST expose ast-grep through a pinned toolchain or package profile with reproducible binary identity.

#### Scenario: Tool identity is reported
- GIVEN a Mantle-provided environment includes ast-grep
- WHEN a repository records ast-grep evidence
- THEN the evidence MUST identify the ast-grep package version and binary identity used for the run.

### Requirement: Structural scan sidecars
r[mantle.ast_grep_structural_rails.sidecar] Mantle SHOULD support ast-grep rule-test and scan sidecars that record command identity, tool identity, rule bundle identity, scan scope, output format, finding summary, and non-claims.

#### Scenario: Valid sidecar is attached to build evidence
- GIVEN a repository produces an ast-grep scan sidecar during a verification workflow
- WHEN Mantle records build or release evidence
- THEN the sidecar MAY be attached as structural verification evidence with its non-claims preserved.

### Requirement: BLAKE3 evidence identity
r[mantle.ast_grep_structural_rails.identity] Mantle MUST use BLAKE3 for Mantle-owned ast-grep sidecar, receipt, scan-input, and rule-bundle identities unless a compatibility field preserves another tool-emitted identifier as metadata.

#### Scenario: Rule bundle changes
- GIVEN a build report references an ast-grep rule bundle identity
- WHEN the rule bundle content changes
- THEN a new BLAKE3 identity MUST be recorded before fresh evidence can supersede the earlier report.

### Requirement: Shell-owned execution
r[mantle.ast_grep_structural_rails.shell_boundary] Mantle MUST keep ast-grep process execution, filesystem reads, and raw output capture in build or CLI shell code.

#### Scenario: Evidence validator checks parsed sidecar
- GIVEN an ast-grep sidecar has already been loaded
- WHEN Mantle validates the sidecar shape
- THEN pure validation MUST NOT spawn ast-grep, inspect the filesystem, or read environment state.

### Requirement: Positive and negative sidecar fixtures
r[mantle.ast_grep_structural_rails.fixtures] Mantle MUST include positive and negative fixtures for ast-grep sidecar validation.

#### Scenario: Malformed sidecar fails closed
- GIVEN a sidecar has a stale bundle hash, wrong tool identity, missing non-claims, unsupported output format, or release overclaim
- WHEN Mantle validates the sidecar
- THEN validation MUST reject it with deterministic diagnostics.

### Requirement: Validation evidence
r[mantle.ast_grep_structural_rails.validation] Mantle MUST validate ast-grep structural-rail packaging with package identity smoke, sidecar fixtures, Cairn gates, and focused verification-evidence checks.

#### Scenario: Packaging change is reviewed
- GIVEN ast-grep package or sidecar validation changes
- WHEN validation evidence is assembled
- THEN the evidence MUST include the pinned tool identity, fixture results, lifecycle gate results, and focused Mantle checks.
