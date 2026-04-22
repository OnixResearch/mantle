# Ownership Review

Status: first-wave boundary review synchronized to main spec assets
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
- `crates/crunch-project/src/refresh.rs` → adapter-only re-export
- `crates/crunch-project/src/upgrade.rs` → adapter-only wrapper over core upgrade error/result

## Touched std workspace source files outside the legacy paths

- `crates/crunch-attestation/src/adapter.rs` → `adapter-only`
  - borrowed/std-facing attestation wrappers translate inputs into owned core
    request types before calling `crunch-attestation-core`
- `crates/crunch-attestation/src/discovery.rs` → `unrelated`
  - std shell file discovery stays here intentionally; this is effectful
    directory/file loading, not first-wave business logic moved back out of core
- `crates/crunch-attestation/src/lib.rs` → `adapter-only`
  - crate-root module wiring and re-exports for the std-facing attestation shell
- `crates/crunch-project/src/attestation.rs` → `adapter-only`
  - reduced to a re-export shim over `attestation_adapter.rs`
- `crates/crunch-project/src/attestation_adapter.rs` → `adapter-only`
  - borrowed/std-facing project-attestation wrapper builds owned
    `ProjectAttestationRequest` values for `crunch-project-core`
- `crates/crunch-project/src/error.rs` → `adapter-only`
  - std-facing error translation keeps `serde_json::Error` and `std::io::Error`
    outside the no-std core boundary
- `crates/crunch-project/src/lib.rs` → `adapter-only`
  - crate-root module wiring and `pub use` exports for the std-facing shell;
    no first-wave project business logic lives here
- `crates/crunch-project/src/mirrors.rs` → `unrelated`
  - touched while the first-wave project-core surface changed, but it remains a
    thin mirror helper re-export and does not reintroduce moved business logic
- `crates/crunch-project/src/refresh_adapter.rs` → `adapter-only`
  - resolver/file/network hashing stays in the std adapter while the core sees
    only owned normalized refresh requests and outcomes
- `crates/crunch-project/src/upgrade_adapter.rs` → `adapter-only`
  - keeps CLI-facing upgrade error translation in std while
    `crates/crunch-project/src/upgrade.rs` stays a re-export shim
- `crates/crunch-project/tests/integration_nickel.rs` → `unrelated`
  - integration coverage for generated Nickel/import behavior; no first-wave
    business logic moved back into std production code
- `src/attest_cmd.rs` → `unrelated`
  - root CLI shell for attestation commands, store access, and output formatting
- `src/project_cmd.rs` → `unrelated`
  - root CLI shell still owns manifest/lock file I/O, user-facing output, and
    command orchestration around the std `crunch-project` API
- `tests/attest_cli.rs` → `unrelated`
  - CLI integration coverage only; no first-wave core logic reintroduced
- `tests/project_cli.rs` → `unrelated`
  - CLI smoke coverage only; no first-wave core logic reintroduced
- `tests/project_refresh_cli.rs` → `unrelated`
  - CLI refresh integration coverage only; no first-wave core logic reintroduced

## Review verdict

- No first-wave attestation foundational business logic remains in the reduced
  std legacy files listed above.
- No first-wave project foundational business logic remains in the reduced std
  legacy files listed above.
- No first-wave business logic was reintroduced in the touched std workspace
  source files derived from the change history outside the legacy paths.
- Remaining first-wave shell-owned logic still intentionally lives in std files
  called out by `workspace-inventory.md` (`discovery.rs`, `refresh_adapter.rs`,
  CLI shells, and related adapters) until a later extraction wave explicitly
  reshapes those APIs.

Reviewer: pi session
