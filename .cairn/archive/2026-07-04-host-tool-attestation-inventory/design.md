## Context

Host-tool-free proof work already has a protected-exec seam, but the boundary is strongest when every allowed host executable is represented as declared data. The core should validate inventory records and execution observations, while the shell computes digests, reads version output, and installs supervisors.

## Decisions

### 1. Inventory records are explicit authority

**Choice:** Each host tool record carries role, absolute path, BLAKE3 digest, optional bounded version text digest, and provenance note.

**Rationale:** Role and digest together prevent name-only or path-only trust.

### 2. Protected exec enforces the inventory

**Choice:** In strict proof paths, observed `execve` and `execveat` targets must match an accepted inventory record or a Mantle-built sandbox transition record.

**Rationale:** Observation-based denial catches hidden fallbacks even when PATH was sanitized.

### 3. Inventory acceptance is not bootstrap proof

**Choice:** Reports clearly state that accepted host tools are declared assumptions, not source-built or independently verified tools.

**Rationale:** This keeps trust-reduction claims honest.

## Risks / Trade-offs

- Digesting and version-probing host tools adds preflight cost.
- Unsupported kernels or seccomp setups will block no-host-tools proof mode rather than silently downgrading.
