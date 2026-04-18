# Design: classify dependency audit findings

## Context

`cargo deny check` is already the right lens for this repo, but the current
result mixes runtime and dev-only findings, first-party-controlled upgrades and
vendored blocked upgrades, and actionable fixes versus long-tail upstream debt.
That makes the output informative but not yet governable.

## Goals / Non-Goals

**Goals**
- keep one checked-in dependency-audit entry point
- classify current findings by ownership and actionability
- prefer real fixes over persistent waivers when this repo controls the fix
- make remaining waivers narrow, reviewable, and revisitable

**Non-Goals**
- remove all third-party risk from the tree in one pass
- hide blocked findings behind broad permanent ignores
- redesign unrelated runtime subsystems just to make the first audit green

## Decisions

### 1. `cargo deny` remains the audit authority

**Choice:** keep `cargo deny check` or a small checked-in wrapper around it as
the canonical dependency audit entry point.

**Rationale:** the tool already reports the vulnerability, unsound,
unmaintained, and yanked classes we care about. The missing piece is repo-local
policy, not a new scanner.

**Implementation:** add checked-in config and docs that tell contributors how
the repo expects deny output to be interpreted.

### 2. Findings are classified before waivers are added

**Choice:** classify each current finding into one of these buckets before any
persistent waiver lands:

- first-party controlled and directly upgradable
- transitive but realistically upgradable from this repo
- vendored or upstream blocked
- dev-only / non-runtime
- accepted temporarily with explicit rationale

**Rationale:** this keeps the repo from treating easy fixes and blocked
upstream debt as the same problem.

**Implementation:** the machine-enforced deny config will carry the narrow
waivers, and a companion checked-in markdown note will carry the human-readable
classification for the current finding set.

### 3. Waivers must be narrow and revisitable

**Choice:** remaining blocked findings get narrow checked-in waivers with the
advisory identifier or class, affected crate, ownership path, rationale, and a
review trigger or expiry condition.

**Rationale:** broad silent ignores would make future audit output meaningless.

**Implementation:** the checked-in deny config and supporting docs will carry
that metadata close to the waiver itself.

## Verification Strategy

- Prove the checked-in audit entry point with a `cargo deny check` transcript
  from the repo's final policy state.
- Prove classification coverage by checking in and reviewing the companion
  finding-classification note alongside the deny config.
- Prove fix-before-waiver behavior by keeping before/after evidence for each
  actionable finding resolved through dependency updates instead of waivers.
- Prove the remaining waivers are narrow by reviewing the final deny config and
  companion notes for per-finding scope, rationale, ownership, and review
  trigger data.

## Risks / Trade-offs

**[Vendor upgrades may be disruptive]** -> Some findings sit behind vendored or
upstream-pinned crates and may not be safely removable in one change.
Mitigation: keep those as explicit narrow waivers with ownership and review
triggers instead of forcing risky drive-by upgrades.

**[Waiver sprawl]** -> Too many permanent exceptions would turn the audit into a
ritual. Mitigation: require classification first and review metadata on every
remaining waiver.
