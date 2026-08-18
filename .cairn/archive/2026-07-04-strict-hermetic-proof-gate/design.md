## Context

Strict hermeticity exists as an execution mode, but proof workflows need a separate eligibility decision that prevents degraded execution evidence from being reused as release or reproducibility proof. The core decision should be deterministic over report data, not over filesystem state or process execution.

## Decisions

### 1. Proof eligibility is a pure classifier

**Choice:** Add a pure classifier over hermeticity mode, typed audit events, closure-resolution status, host-tool/protected-exec status, and proof workflow kind.

**Rationale:** The same eligibility result can be unit tested with in-memory fixtures and reused by release, deterministic-release, self-hosting, and witness shells.

### 2. Strict mode is required for proof admission

**Choice:** Proof-mode workflows admit evidence only when the run selected strict hermeticity and no unapproved degraded hermeticity event exists.

**Rationale:** Practical and impure modes are useful diagnostic tools, but accepting them as proof evidence would make hidden host dependencies indistinguishable from declared inputs.

### 3. Downgrades are explicit non-claims

**Choice:** If a proof command runs in practical or impure mode, the report names the narrower diagnostic claim and marks release/reproducibility proof admission as blocked.

**Rationale:** Operators still get useful failure context without weakening the meaning of proof labels.

## Risks / Trade-offs

- Existing loose workflows may need explicit `--strict-hermetic` or report weaker evidence.
- Some unsupported platforms will move from warning to proof-blocked until their sandbox guarantees are modeled.
