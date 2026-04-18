# Classify dependency audit findings

## Why

The audit showed that `cargo deny check` currently reports a mix of problems:
upgradable security issues, unsoundness reports, yanked crates, unmaintained
upstream crates, and findings that arrive through vendored or otherwise blocked
upgrade paths.

Right now those findings are real, but the repo does not have one checked-in
policy that separates:

- issues we can fix directly in this repo,
- issues we inherit through vendored or upstream-pinned code,
- issues that are dev-only or non-runtime,
- and issues that remain blocked but intentionally accepted for now.

We need a dependency-audit change that makes the current risk explicit and
keeps future audit noise from being hand-waved away.

## What Changes

- define a checked-in dependency-audit policy anchored on a reproducible audit
  entry point
- classify current findings by ownership and actionability: first-party,
  transitive-upgradable, vendored/upstream-blocked, dev-only, or intentionally
  waived with review metadata
- remediate directly actionable findings where this repo controls the fix, and
  keep the remaining blocked findings behind narrow documented waivers instead
  of ad hoc acceptance

## Capabilities

### New Capabilities

- `checked-in-dependency-audit`: contributors get one reproducible dependency
  audit policy from the repo
- `classified-audit-findings`: current and future audit hits are grouped by
  ownership and actionability instead of one undifferentiated red wall
- `waiver-tracking`: blocked findings carry rationale and review triggers

## Impact

- **Files**: dependency-audit config, documentation, `Cargo.lock`, and any
  manifests or vendored crates touched by upgrades or narrow waivers
- **APIs**: no intended runtime API surface change
- **Dependencies**: may update direct and transitive crate versions or add
  narrow checked-in waivers for blocked findings
- **Testing**: needs reproducible dependency-audit verification and evidence
  that direct fixes were preferred over waivers where possible

## Verification

A reviewer should expect this change to land with:

- one checked-in dependency-audit entry point anchored on `cargo deny check`
  or a checked-in wrapper around it
- a triage of the current audit findings that separates actionable first-party
  fixes from blocked vendored/upstream findings and from dev-only exposure
- direct or tractable transitive fixes applied where this repo controls the
  upgrade path
- narrow waiver records for the remaining blocked findings, with rationale,
  ownership, and a review trigger or expiry condition
- an audit transcript showing no unclassified findings remain in the checked-in
  policy

## Non-Goals

- promise that every vendored or upstream-pinned finding can be eliminated in a
  single change
- replace vendored subsystems solely to remove one advisory when a narrower
  policy-tracked mitigation is more realistic
- treat a blocked dev-only or vendored finding as equivalent to an immediately
  upgradable first-party runtime issue
