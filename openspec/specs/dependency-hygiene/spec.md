# dependency-hygiene Specification

## Purpose
TBD - created by archiving change classify-dependency-audit-findings. Update Purpose after archive.
## Requirements
### Requirement: Dependency audit policy is checked in

The repo MUST keep a checked-in dependency-audit policy for the Cargo
workspace.

That policy MUST define one reproducible audit entry point anchored on
`cargo deny check` or on a checked-in wrapper around `cargo deny check`.

The policy MUST cover at least vulnerability, unsound, unmaintained, and yanked
findings.

#### Scenario: New unclassified advisory fails the audit

- GIVEN the dependency graph introduces a new advisory or audit finding not
  covered by the checked-in policy
- WHEN a contributor runs the repo's dependency-audit entry point
- THEN the command exits non-zero
- AND the output shows the new finding instead of silently accepting it

### Requirement: Findings are classified by ownership and actionability

Current dependency audit findings MUST have a reviewable checked-in
classification by ownership and actionability.

The classification MUST separate at least these buckets:

- first-party actionable,
- transitive but realistically upgradable from this repo,
- vendored or upstream-blocked,
- and dev-only or non-runtime exposure.

The classification MAY live in checked-in deny config comments or in a
companion repo document, but it MUST be versioned with the repo.

#### Scenario: Contributor can review the current finding split

- GIVEN a contributor inspects the current dependency audit policy and notes
- WHEN they review the checked-in classification
- THEN they can tell which findings are actionable from this repo
- AND they can tell which findings remain blocked behind vendored or upstream
  constraints
- AND they can tell which findings are dev-only or non-runtime

### Requirement: Actionable first-party dependency findings prefer remediation

Actionable dependency findings MUST prefer remediation over persistent waiver
when this repo controls a realistic fix path.

If a finding can be addressed by a direct dependency update, lockfile update,
or another dependency change controlled in this repo without forking a vendored
subsystem, the repo MUST prefer that fix before adding a persistent waiver.

#### Scenario: Upgradable first-party finding is fixed instead of waived

- GIVEN a dependency audit finding on a crate version that this repo can update
  directly or tractably through its manifests or lockfile
- WHEN the finding is triaged
- THEN the chosen resolution is a dependency fix
- AND the repo does not add a persistent waiver for that finding first

### Requirement: Remaining blocked findings carry explicit narrow waiver metadata

Remaining blocked dependency findings MUST carry explicit checked-in waiver
metadata at per-finding granularity.

A waiver MUST target a specific advisory identifier or other finding identifier
plus the affected crate. A blanket category-level suppression on its own MUST
NOT satisfy this requirement.

That metadata MUST include the advisory identifier or finding class, the
affected crate, the ownership path or scope (`vendored`, `upstream-blocked`,
`dev-only`, or equivalent), a rationale, and a review trigger or expiry
condition.

#### Scenario: Vendored blocked finding remains narrow and reviewable

- GIVEN a dependency audit finding that remains blocked behind vendored or
  upstream-pinned code
- WHEN the repo keeps a waiver for it
- THEN the waiver record identifies the specific finding and affected crate
- AND it records why the finding remains blocked
- AND it records when or why the waiver must be revisited
- AND the waiver is narrower than a blanket category suppression

