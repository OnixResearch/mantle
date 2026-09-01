## Context

The focused command `nix run .#tigerstyle -- check -- -p crunch-store`
reports 183 findings across 13 source files. The repository check previously
reported 139 location-backed findings across the same files. Both results show
strict source debt, not an accepted baseline.

`crunch-store` is an imperative store shell around deterministic lifecycle,
retention, overlay, and provenance cores. A structural lint repair can become a
semantic change if assertions replace input errors, helpers reorder effects,
collection reservations remove bounds, or interface cleanup changes public
contracts.

## Portfolio Search Contract

- **Goal:** make focused and repository Tiger Style checks pass for
  `crunch-store` with unchanged accepted behavior and no lint allowance.
- **Completion evidence:** zero focused findings, a passing repository Tiger
  derivation, passing positive and negative `crunch-store` tests, strict
  Clippy, formatting, and a full-check result that advances beyond this gate.
- **False completion:** disabled lints, reduced package or target scope, warning
  budgets, finding baselines, panics replacing typed input errors, deleted
  negative coverage, or a check that stops before `crunch-store`.
- **Audit risks:** effect reordering, assertion-triggered denial-of-service,
  lost overflow or collection bounds, changed canonical identities, public API
  drift, and policy leakage from cores into I/O helpers.
- **Budget:** current repository and pinned Tiger input only; 13 source files;
  no new runtime authority or dependency; at most eight focused Tiger rounds;
  local builders for final Nix evidence.
- **Allowed outcomes:** validated, exact later blocker, exhausted round budget,
  or user decision required for an incompatible public contract.

## Approach Registry

### Family: invariant-preserving local repair

**Mechanism:** add meaningful assertions after existing validation, rename
quantities, reserve from checked bounds, and decompose conditions into named
booleans or branches.

**Claim:** most findings can close without changing data flow or effects.

**Artifact:** narrow source edits and unchanged positive/negative test results.

**State:** active.

### Family: functional helper extraction

**Mechanism:** split long functions by existing observation, validation,
planning, and mutation phases. Pass explicit normalized facts between helpers.

**Claim:** function-length and assertion-density findings can close while
preserving functional-core and imperative-shell ownership.

**Artifact:** helpers under 70 lines with explicit invariants and tests at the
same public boundary.

**State:** active where local repair is insufficient.

### Family: policy suppression

**Mechanism:** add `allow` attributes, budgets, baselines, or narrower check
scope.

**Claim:** the check can report success without source repair.

**Blocker:** this violates the task, Octet policy, and the accepted full-check
requirement.

**State:** falsified and prohibited.

## Decisions

### Decision: assertions state internal facts only

**Choice:** Assert facts already guaranteed by type construction, prior checked
validation, bounded loop state, or successful effect results. Keep malformed or
untrusted input on explicit `Result` errors.

**Rationale:** Tiger assertions must expose impossible internal states. They
must not create new process-abort behavior for ordinary rejected input.

### Decision: preserve effect order during extraction

**Choice:** Extract contiguous phases without moving filesystem, service,
network, signing, or mutation calls across validation and authority checks.

**Rationale:** Function length is not authority to reorder the imperative
shell.

### Decision: preserve public compatibility

**Choice:** Use named input records for private helpers. Keep public wrappers
and existing types unless the strict checker requires a public shape change;
then add a compatible wrapper and test it.

**Rationale:** Lint cleanup does not authorize a store API redesign.

### Decision: validate in ascending cost

**Choice:** Run formatting and focused tests after each file family, then the
focused Tiger command. Run repository Tiger and full Nix checks only after the
focused result is clean.

**Rationale:** Short feedback cycles reduce correlated repair errors and avoid
rebuilding the full workspace for local mistakes.

## Risks / Trade-offs

- Meaningful assertions can reveal pre-existing invalid internal state. Tests
  must cover accepted and rejected paths before integration.
- Helper extraction can increase type count but reduces mixed ownership inside
  long functions.
- The full check can expose a later independent blocker after `crunch-store`
  becomes clean.
