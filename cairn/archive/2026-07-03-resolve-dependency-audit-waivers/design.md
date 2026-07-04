## Context

The current audit policy intentionally records residual upstream-blocked risks, including `quick-xml` through `object_store` and `proc-macro-error2` through `oci-spec`/`getset`. These should not become permanent background noise.

This change separates two outcomes: retire a waiver through dependency movement, or keep it with sharper evidence and regression protection.

## Decisions

### 1. Policy file remains mandatory evidence

**Choice:** Every audit command used as evidence must pass `--config deny.toml` or otherwise prove it loaded the checked-in policy.

**Rationale:** Default cargo-deny output is noisy and can hide the real advisory set.

### 2. Upgrade only the affected path first

**Choice:** Prefer minimal lock/manifest updates that affect the waived advisory path before broad workspace updates.

**Rationale:** Smaller dependency movement is easier to review and less likely to perturb proof paths.

### 3. Waivers need unblock conditions

**Choice:** Any remaining waiver must name the transitive path, upstream version constraint, owner when identifiable, and the exact condition that would allow removal.

**Rationale:** A waiver without a removal condition is not auditable.

### 4. Compile evidence follows audit movement

**Choice:** After changing dependency versions or features, run focused compile checks for the crates that transitively consume the moved dependencies.

**Rationale:** Passing the audit alone does not prove Mantle still builds.

## Risks / Trade-offs

- Some upstream stacks may require larger version migrations than this change should take.
- Advisory databases can change during the work; evidence must include same-run command output.
- Dependency updates can perturb Cargo-free source material and should be checked against fixed-point proof needs.
