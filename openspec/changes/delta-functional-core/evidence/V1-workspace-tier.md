Evidence-ID: delta-functional-core-v1-workspace-tier
Task-ID: V1
Artifact-Type: verification
Covers: architecture.nostd.core.workspace.tier.visible, functional.core.dedicated.nostd.crates.third.wave, functional.core.dedicated.nostd.crates.first.wave, functional.core.dedicated.nostd.crates.second.wave
Status: complete
Date: 2026-04-22

# V1 verification packet

## Command transcript

Executed command:

- `openspec validate delta-functional-core`
  - observed output: `Change 'delta-functional-core' is valid`

Supplemental adopted-core checker evidence captured the same session:

- `python3 scripts/no_std_core_checks.py scope`
  - `scope check OK`
- `python3 scripts/no_std_core_checks.py api-shape`
  - `API shape check OK`
- `python3 scripts/no_std_core_checks.py ownership`
  - `ownership check OK: crates/crunch-delta/src/lib.rs, crates/crunch-delta/src/manifest.rs, crates/crunch-delta/src/substitution.rs, crates/crunch-shell/src/adapter.rs, crates/crunch-shell/src/lib.rs, crates/crunch-shell/src/types.rs, src/release_cmd.rs, src/release_evidence.rs`
- `python3 scripts/no_std_core_checks.py deps`
  - `dependency allowlist OK: ... crunch-delta-core ...`

## Workspace-tier inspection

### `Cargo.toml`

- workspace members still include the first-wave and second-wave adopted cores:
  - `crates/crunch-attestation-core`
  - `crates/crunch-project-core`
  - `crates/crunch-shell-core`
  - `crates/crunch-release-core`
- workspace members now also include `crates/crunch-delta-core`
- tigerstyle default scope also names `-p crunch-delta-core`

### `crates/crunch-delta-core/src/lib.rs`

- file begins with `#![no_std]`
- file declares `extern crate alloc`
- module docs explicitly say `crunch-delta` stays std-facing and owns castore/store/network conversion, manifest probing, and substitution orchestration around the core crate

### `crates/crunch-delta/src/lib.rs`

- crate root still exports the std-facing delta façade
- public surface is `crunch-delta` itself, not a bare `crunch-delta-core` passthrough
- façade still owns manifest building and substitution exports

### `openspec/specs/functional-core/validation/adopted-core-inventory.toml`

- includes a dedicated `[[core]]` entry for `package = "crunch-delta-core"`
- records `crate_dir = "crates/crunch-delta-core"`
- records required std-adapter files:
  - `crates/crunch-delta/src/lib.rs`
  - `crates/crunch-delta/src/manifest.rs`
  - `crates/crunch-delta/src/substitution.rs`

### `openspec/specs/functional-core/evidence/ownership-review.md`

- records `crates/crunch-delta/src/lib.rs` as `adapter-only`
- records `crates/crunch-delta/src/manifest.rs` as `adapter-only`
- records `crates/crunch-delta/src/substitution.rs` as `adapter-only`
- review verdict explicitly says delta planning/protocol business logic remains in `crunch-delta-core`

## Proposal-stage transcript linkage

This packet intentionally references the proposal-stage validation transcript required by the task:

- `openspec/changes/delta-functional-core/evidence/proposal-validation-2026-04-22.md`

## Verdict

V1 satisfied on 2026-04-22.

The workspace tier now names `crunch-delta-core` alongside the earlier adopted cores, keeps `crunch-delta` as the std adaptor layer, and records the delta adaptor files in both the adopted-core inventory and the ownership review.
