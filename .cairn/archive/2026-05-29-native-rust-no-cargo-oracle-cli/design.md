# Design: Cargo-free Rust CLI mode

## Context

A real alternative needs a command users can run that does not rely on Cargo as planner or build orchestrator.

## Decisions

### 1. Add explicit no-Cargo mode

**Choice:** Provide a CLI switch or command such as `rust-plan --no-cargo-oracle --execute-topology` that routes through native planner and executor only.

**Rationale:** Operator intent and evidence must distinguish oracle-assisted development from Cargo-free operation.

### 2. Fail if Cargo is invoked in no-Cargo mode

**Choice:** Guard no-Cargo mode with command resolution/audit checks and tests using a failing Cargo shim.

**Rationale:** Accidental Cargo fallback would invalidate the claim.

### 3. Emit bounded claims and non-claims

**Choice:** JSON receipts state whether the run was Cargo-free, which surfaces were unsupported, and what compatibility class was proven.

**Rationale:** Users need honest evidence, not broad Cargo parity claims.

## Risks / Trade-offs

- Initial CLI may block many projects until native planner coverage expands.
- Existing oracle-assisted workflows must remain available for comparison.
