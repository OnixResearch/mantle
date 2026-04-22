# Ownership Review

Status: partial implementation review for first-wave extraction scaffolding
Date: 2026-04-22

## Legacy std paths reduced to adapter-only form

### `crunch-attestation`

- `crates/crunch-attestation/src/error.rs` → adapter-only re-export
- `crates/crunch-attestation/src/digest.rs` → adapter-only re-export
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

## Touched std workspace source files outside the legacy paths

- `crates/crunch-project/src/refresh.rs` → `unrelated`
  - removed one now-unused test import after moving foundational types to
    `crunch-project-core`

## Review verdict

- No first-wave attestation foundational business logic remains in the reduced
  std legacy files listed above.
- No first-wave project foundational business logic remains in the reduced std
  legacy files listed above.
- Remaining first-wave business logic still intentionally lives in std-owned
  files called out by `workspace-inventory.md` (`canonical.rs`, `policy.rs`,
  `release.rs`, `refresh.rs`, `upgrade.rs`, and related
  adapters) until their APIs are reshaped for the stricter core boundary
  rules.

Reviewer: pi session
