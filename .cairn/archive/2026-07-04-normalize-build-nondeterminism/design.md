## Context

Deterministic proof labels require control over more than declared file inputs. Time, locale, umask, temp roots, host/user names, random seeds, and output traversal order can all perturb bytes or receipts. Normalization should be declared as data and enforced in the shell.

## Decisions

### 1. Normalization policy is receipt-bound

**Choice:** Strict builds and proof envelopes record the selected normalization policy for time, locale, timezone, umask, temp roots, host/user metadata, randomness where modeled, and ordering.

**Rationale:** Equivalent outputs can only be compared honestly when the controls are visible.

### 2. Unsupported controls block strong claims

**Choice:** If Mantle cannot enforce a requested normalization control, the build may be diagnostic-only but cannot satisfy strict proof or strong build-correctness claims.

**Rationale:** Silent partial normalization creates false confidence.

### 3. Perturbation fixtures prove the boundary

**Choice:** Tests vary ambient host settings and assert either identical admitted evidence or deterministic blockers.

**Rationale:** Positive and negative perturbations catch regressions that simple clean builds miss.

## Risks / Trade-offs

- Some tools may still embed nondeterminism internally; Mantle should report divergence rather than claiming to fix tool behavior.
- Bounding parallelism where output order matters can reduce performance in strict proof paths.
