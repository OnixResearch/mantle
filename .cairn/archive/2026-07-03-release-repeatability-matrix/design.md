## Context

Current global reproducibility reports can admit a digest-bound release universe, but the strongest current release evidence is still produced by specific proof runs. A repeatability matrix turns that into a controlled experiment over declared axes while keeping final admission in the existing global evaluator.

## Decisions

### 1. Matrix planning is pure and digest-bound

**Choice:** Represent the matrix as a deterministic profile: release id, artifact surfaces, run count, cache mode, store isolation, user/env/temp controls, host class, and expected output digest set.

**Rationale:** A pure plan lets tests assert the full matrix before running expensive builds, and its BLAKE3 digest becomes part of the evidence boundary.

### 2. Execution cells use isolated roots

**Choice:** Each cell gets a fresh output root and store root unless the cell explicitly tests substitution/reuse.

**Rationale:** Fresh roots prevent accidental reuse from looking like reproducibility. Reuse experiments stay explicit blockers or separate evidence classes.

### 3. Mismatches are first-class evidence

**Choice:** Matrix reports preserve mismatched, missing, reused-store, and unsupported cells instead of truncating to pass/fail.

**Rationale:** Useful reproducibility evidence must explain which axis failed and why; global admission can then block only the affected surfaces.

## Risks / Trade-offs

- Full matrices are expensive; provide small smoke profiles and larger release profiles.
- Different hosts may lack the same sandbox features; unsupported cells must be recorded rather than silently skipped.
