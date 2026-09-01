## ADDED Requirements

### Requirement: Build layer maintains strict Tiger Style conformance

r[build_correctness.tiger_conformance] Mantle MUST keep `crunch-build` and `crunch-rustc-wrapper` within the complete pinned Tiger Style policy without lint allowances, warning budgets, finding baselines, target-scope reductions, or weaker full-check enforcement, while preserving derivation identity, canonical build inputs, content-addressed planning, typed profile admission, scheduler effect order, publication authority, and public compatibility.

#### Scenario: strict build check accepts both packages

GIVEN build-layer source uses checked arithmetic, bounded non-recursive processing, meaningful invariants, typed error propagation, decomposed conditions, and named interfaces
WHEN the focused package check and repository Tiger Style check run
THEN both MUST report zero `crunch-build` and `crunch-rustc-wrapper` findings without an allowance or suppressed target
AND positive and negative package tests, strict Clippy, and formatting MUST pass.

#### Scenario: structural repair would change build meaning

GIVEN a proposed lint repair changes canonical structured-attribute bytes, content-addressed output selection, profile admission, registry or scheduler order, no-replace publication, or public behavior without compatibility
WHEN the repair is reviewed or tested
THEN Mantle MUST reject the repair even if the Tiger Style command exits successfully
AND the finding MUST remain actionable until a semantics-preserving repair passes.

#### Scenario: full check advances beyond the build gate

GIVEN focused build validation and the repository Tiger Style check pass
WHEN local-builder and ordinary `nix flake check -L` run
THEN neither run MUST fail on a `crunch-build` or `crunch-rustc-wrapper` Tiger Style finding
AND any later independent failure MUST remain an exact blocker without disabling or downgrading its gate.
