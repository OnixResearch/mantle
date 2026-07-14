# Stateful workspace implementation inventory

## Pre-change seam classification

| Seam | Owner | Prior state | Workspace integration |
|---|---|---|---|
| Sandbox scratch roots and mounts | `vendor/snix-build` `BuildRequest`, `SandboxSpec`, bwrap | Ephemeral per build | Explicit immutable read-only or mutable writable mount at a validated stable guest path; runtime host paths are skipped from serialization. |
| Local retained state | `crates/crunch-build` | Absent | Private state-dir subtree with no-follow traversal, exclusive lock, atomic lease record, scrub, quota, quarantine, snapshot, and retention shell. |
| Remote worker ownership | `src/remote_build.rs` coordinator state | Durable job/attempt/fence state only | Durable endpoint workspace registrations and compatibility/job/attempt/fence-bound workspace leases use the pure transition core. |
| Action identity and admission | `crunch-glue` conversion and `crunch-build` orchestration | Derivation identity plus strong cache lookup | Typed policy is encoded in the hashed derivation; immutable snapshot refs affect identity. Mutable builds bypass strong cache lookup and remote job attach/redelivery candidates. |
| Secrets and host paths | Sandbox environment policy and build reports | No retained-state scanner | Bounded file scan detects configured markers and host-path material, scrubs exact sensitive paths, quarantines unsafe state, and omits runtime host paths and content from reports/transport. |
| Store GC roots | `crunch-store` | Store-output lifecycle | Retained workspaces are not store GC roots. They have a separate deterministic retention policy; admitted output objects retain the existing store lifecycle. |
| Structured evidence | `crunch-pipeline` and `src/build_report.rs` | Build/environment/network evidence | `workspace_reports` records actual mode, warm-state use, claim downgrade, cleanup, snapshot refs, and clean/warm output-set digests. |

## Boundary decisions

- Pure bounded mode, lease, quota, scrub, snapshot, retention, claim, and clean-comparison decisions live in `crates/crunch-build/src/workspace.rs`.
- Filesystem I/O, locks, atomic records, no-follow operations, CAS ingestion, and service orchestration live in `workspace_shell.rs`.
- Mutable history may produce admitted output objects, but cannot satisfy or publish a strong shared action-result candidate.
- A clean comparison is separate evidence and never changes the original mutable execution's hermeticity or claim class.
