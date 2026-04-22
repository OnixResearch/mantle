# Ownership Review

Status: partial implementation review for first-wave extraction scaffolding
Date: 2026-04-22

## Legacy std paths reduced to adapter-only form

### `crunch-attestation`

- `crates/crunch-attestation/src/canonical.rs` → adapter-only re-export
- `crates/crunch-attestation/src/digest.rs` → adapter-only re-export
- `crates/crunch-attestation/src/error.rs` → adapter-only re-export
- `crates/crunch-attestation/src/policy.rs` → adapter-only re-export
- `crates/crunch-attestation/src/release.rs` → adapter-only re-export
- `crates/crunch-attestation/src/schema.rs` → adapter-only re-export
- `crates/crunch-attestation/src/version.rs` → adapter-only re-export

### `crunch-project`

- `crates/crunch-project/src/manifest.rs` → adapter-only re-export
- `crates/crunch-project/src/lock.rs` → adapter-only re-export
- `crates/crunch-project/src/version.rs` → adapter-only re-export
- `crates/crunch-project/src/merge.rs` → adapter-only re-export
- `crates/crunch-project/src/generate.rs` → adapter-only re-export
- `crates/crunch-project/src/drift.rs` → adapter-only re-export
- `crates/crunch-project/src/mirrors.rs` → adapter-only re-export
- `crates/crunch-project/src/upgrade.rs` → adapter-only wrapper over core upgrade error/result

## Touched std workspace source files outside the legacy paths

- `crates/crunch-project/src/lib.rs` → `adapter-only`
  - crate-root module wiring and `pub use` exports for the std-facing shell;
    no first-wave project business logic lives here
- `crates/crunch-project/src/refresh.rs` → `unrelated`
  - removed one now-unused test import after moving foundational types to
    `crunch-project-core`; file remains a thin std re-export surface
- `crates/crunch-project/src/upgrade_adapter.rs` → `adapter-only`
  - keeps CLI-facing error translation in std while `crates/crunch-project/src/upgrade.rs`
    is reduced to re-exports only
- `src/project_cmd.rs` → `unrelated`
  - root CLI shell still owns manifest/lock file I/O, user-facing output, and
    command orchestration around the std `crunch-project` API

## Review verdict

- No first-wave attestation foundational business logic remains in the reduced
  std legacy files listed above.
- No first-wave project foundational business logic remains in the reduced std
  legacy files listed above.
- No first-wave project business logic was reintroduced in the touched std
  files outside the legacy paths (`crates/crunch-project/src/lib.rs`,
  `crates/crunch-project/src/refresh.rs`,
  `crates/crunch-project/src/upgrade_adapter.rs`, and `src/project_cmd.rs`).
- Remaining first-wave business logic still intentionally lives in std-owned
  files called out by `workspace-inventory.md` (`canonical.rs`, `policy.rs`,
  `release.rs`, `refresh.rs`, and related adapters) until their APIs are
  reshaped for the stricter core boundary rules.

Reviewer: pi session
