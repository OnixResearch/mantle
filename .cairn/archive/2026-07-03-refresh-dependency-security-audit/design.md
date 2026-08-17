## Context

The repo already has dependency-audit gotchas: a root `deny.toml`, private vendored crates, and upstream-blocked virtualization stack advisories. A useful audit must preserve that context and avoid treating a default cargo-deny wall of noise as meaningful evidence.

## Decisions

### 1. Checked-in policy is required

**Choice:** Run the audit with the repo's checked-in policy and record the policy path or digest.

**Rationale:** Default tool config can produce misleading results that obscure the real advisory set.

### 2. Findings are classified

**Choice:** Each remaining finding should be classified as fixed, accepted waiver, upstream-blocked, action-required, or tooling/config issue.

**Rationale:** Operators need to know whether a finding is newly actionable.

### 3. No clean claim without clean evidence

**Choice:** If any advisory or license finding remains, the final summary must state the narrower classification instead of claiming the audit is clean.

**Rationale:** This follows proof-before-claim and keeps waivers visible.

## Risks / Trade-offs

- Updating dependency pins can cascade into large rebuilds or vendored changes.
- Some advisories may remain blocked by upstream dependency stacks.
- Audit tools may change output schema and require policy updates before useful results are available.
