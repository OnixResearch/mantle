## ADDED Requirements

### Requirement: Store shell maintains strict Tiger Style conformance

r[store_lifecycle.tiger_conformance] Mantle MUST keep `crunch-store` within the complete pinned Tiger Style policy without lint allowances, warning budgets, finding baselines, target-scope reductions, or weaker full-check enforcement, while preserving typed rejection behavior, bounded resource policy, functional-core ownership, shell authority, public compatibility, and effect order.

#### Scenario: strict store check accepts the complete package

GIVEN `crunch-store` source and tests use meaningful invariants, explicit quantity units, bounded collection growth, decomposed conditions, narrow functions, and named interfaces
WHEN the focused package check and repository Tiger Style check run
THEN both MUST report zero `crunch-store` findings without an allowance or suppressed target
AND positive and negative store tests, strict Clippy, and formatting MUST pass.

#### Scenario: structural repair would change store meaning

GIVEN a proposed lint repair replaces a typed external-input error with a panic, removes a bound, reorders validation and effects, changes canonical identity, moves policy into the shell, changes a public contract without compatibility, or deletes negative coverage
WHEN the repair is reviewed or tested
THEN Mantle MUST reject the repair even if the Tiger Style command exits successfully
AND the finding MUST remain actionable until a semantics-preserving repair passes.

#### Scenario: full check advances beyond the store gate

GIVEN focused store validation and the repository Tiger Style check pass
WHEN local-builder and ordinary `nix flake check -L` run
THEN neither run MUST fail on a `crunch-store` Tiger Style finding
AND any later independent failure MUST remain an exact blocker without disabling or downgrading its gate.
