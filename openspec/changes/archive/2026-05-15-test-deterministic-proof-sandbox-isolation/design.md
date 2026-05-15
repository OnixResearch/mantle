## Context

The previous change added a proof sandbox profile identity and routes deterministic proof rebuilds through a bwrap-compatible command. The current local environment may not provide real `bwrap`, so tests use a fake runner. That fake runner should still be useful: it can deterministically inspect the bwrap argument envelope and simulate isolation-denial cases that would otherwise require a host-specific sandbox setup.

## Goals / Non-Goals

**Goals:**
- Add negative/structural tests for deterministic proof sandbox isolation.
- Ensure host-only proof recipes fail closed before proof receipt creation.
- Ensure proof sandbox invocations do not include main rebuild output or reused proof stores.
- Ensure no-network policy remains checkable.

**Non-Goals:**
- Prove every Linux namespace/bwrap kernel behavior in unit tests.
- Convert ordinary `release reproduce` rebuilds to sandboxed execution.
- Broaden deterministic proof claims beyond named release artifacts and recorded profile assumptions.

## Decisions

### 1. Use a policy-checking fake bwrap for CI-safe isolation regressions

**Choice:** Extend the fake bwrap helper in CLI tests so it records/inpects arguments and can deny test recipes that reference an undeclared host path.

**Rationale:** The agent host currently may lack real `bwrap`, but argument-level regressions are still important and deterministic.

**Alternative:** Require a real bwrap integration test. Rejected as the only gate because it would be unavailable in this environment; a future optional test may still be added.

### 2. Check the command envelope, not just receipt JSON

**Choice:** Assert the bwrap invocation includes no host-network opt-in and does not bind the main rebuild output into proof runs.

**Rationale:** A receipt profile can look supported even when the executor command was weakened by a refactor.

## Risks / Trade-offs

**Fake runner is not a kernel sandbox** → Mitigate by limiting the claim to regression evidence for Mantle's invocation construction and fail-closed plumbing, while keeping docs/specs bounded.

**Overfitting to bwrap argument order** → Assert stable invariants and forbidden options/paths rather than every argument.
