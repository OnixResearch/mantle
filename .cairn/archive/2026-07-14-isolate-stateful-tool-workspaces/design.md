## Context

Mantle's strong correctness model assumes declared immutable inputs, explicit sandbox/network policy, and admitted content-addressed outputs. The sandbox shell uses temporary build roots, while shared cache and remote transfer operate on declared object identities. No current contract defines how mutable tool-internal incremental state may survive between executions or what claims must be downgraded when it does.

The design must support useful warm execution without making mutable history an ambient input to strong action reuse.

## Decisions

### 1. Define three explicit workspace modes

**Choice:** Policy supports `none`, `immutable-snapshot`, and `mutable-session`. `none` starts without retained tool state. `immutable-snapshot` mounts declared read-only content-addressed objects. `mutable-session` mounts one shell-owned writable leased workspace.

**Rationale:** Immutable declared cache data and mutable execution history have fundamentally different identity and trust semantics.

### 2. Make immutable snapshots ordinary action inputs

**Choice:** An immutable snapshot has a canonical manifest/object ref, compatibility metadata, bounded size, and stable mount declaration. Its ref participates in action identity and normal CAS/transfer/admission behavior.

**Rationale:** Once immutable and declared, cache content is not ambient state.

### 3. Scope mutable workspaces narrowly

**Choice:** A mutable workspace lease binds workspace id, worker id, tenant/authority class, action compatibility digest, toolchain refs, stable sandbox mount path, current job/attempt/fence, quota policy, creation generation, and retention class. Reuse requires every compatibility and authority field to match.

**Rationale:** Worker-local history must not leak across unrelated actions, tools, tenants, or stale attempts.

### 4. Downgrade mutable-state claims

**Choice:** An execution that reads a mutable session workspace may produce normally admitted output objects, but it cannot publish or satisfy a strong shared action result solely from that run. Strong reuse requires either `none`/`immutable-snapshot` execution or a separate clean rebuild under equivalent declared inputs whose admitted output set matches.

**Rationale:** Output hashing detects what was produced but does not prove mutable history was irrelevant.

### 5. Virtualize the workspace path

**Choice:** The sandbox mounts an accepted workspace at a stable declared guest path independent of worker host paths. Absolute host paths never enter action identity or tool-visible configuration. Path traversal, symlink escape, and incompatible embedded-host-path scans fail policy.

**Rationale:** Stable guest paths reduce tool breakage without making host paths part of portable identity.

### 6. Plan leases and cleanup in a pure core

**Choice:** Pure functions decide mode admission, compatibility, lease transitions, quota accounting, retention, scrub requirements, quarantine, and claim classification. The shell creates directories, locks leases, mounts paths, scans content, snapshots immutable objects, and removes or quarantines state.

**Rationale:** Stateful storage orchestration remains testable through deterministic plans.

### 7. Fail closed on uncertain cleanup or authority

**Choice:** Secret descriptors and policy-defined sensitive paths are excluded or scrubbed before reuse/snapshot. Failed scrub, unknown ownership, stale fence, concurrent lease conflict, or path escape quarantines the workspace and blocks reuse. Cleanup failure is reported separately from the completed build result but blocks any claim requiring successful cleanup.

**Rationale:** Convenience state must not become a persistence or cross-tenant exfiltration channel.

### 8. Bound retention and garbage collection

**Choice:** Typed policy limits workspace count, bytes, files, age generations, idle leases, snapshots, and quarantine. Retention uses explicit roots/leases and deterministic eviction plans; wall-clock observations are shell facts supplied to the core.

**Rationale:** Mutable caches otherwise grow without CAS-style natural identity and retention.

## Functional Core / Imperative Shell

- **Core**: mode validation, compatibility digest construction, lease transition, claim classification, quota and retention plans, scrub/snapshot admission, path-policy validation, and stable diagnostics.
- **Shell**: filesystem creation and locks, sandbox mounts, worker/coordinator persistence, content scans, secret scrubbing, snapshot ingestion, clocks, deletion/quarantine, and report rendering.

## Risks / Trade-offs

- Mutable workspaces improve warm performance but intentionally weaken portable reuse claims.
- Clean-rebuild comparison is expensive; it is opt-in proof evidence rather than an implicit step for every warm build.
- Tool-specific cache formats remain opaque and may still be invalid across tool versions despite compatibility metadata.
- Quarantine consumes disk until explicit bounded cleanup succeeds.
