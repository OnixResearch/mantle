## Why

`mantle.offlineCargoPackage` already has an explicit `vendor_src` input, but `mantle import cargo` still blocks ordinary registry or git dependencies as unsupported even when a workspace has a reviewable vendored source tree. That leaves the near-term offline project-build lane useful mostly for path-only fixtures, while real Rust workspaces still need hand-written Nickel or a separate vendoring workflow.

Mantle should accept vendored Cargo dependency material only when it is declared, lock-bound, checksum-verified, and passed through the same explicit source/input boundary as package sources and toolchains. It should not run `cargo vendor`, discover ambient Cargo caches, or silently permit network access during import.

## What Changes

- Teach `mantle import cargo --plan` to detect supported vendored Cargo source configuration and report deterministic vendor-source facts alongside package, binary, and lockfile facts.
- Validate vendored package entries against `Cargo.lock`, Cargo checksum metadata, and bounded source-layout rules before the generated project can be applied.
- Generate `.mantle/inputs.ncl` and `mantle-project.ncl` entries that pass `vendor_src` / `vendor_name` into `mantle.offlineCargoPackage` when vendored material is accepted.
- Preserve fail-closed blockers for unsupported registry/git material, stale vendored checksums, missing lockfile entries, ambiguous package versions, and unsafe vendored paths.
- Record import-plan non-claims so vendored import support is not presented as network vendoring, full Cargo compatibility, or Cargo-free execution.

## Impact

- **Files**: `src/cargo_import.rs`, `lib/offline_cargo.ncl`, `tests/cargo_import_cli.rs`, `tests/offline_cargo_project.rs`, `tests/rust_compatibility_rail.rs`, docs for offline Cargo import, and this Cairn spec delta.
- **Testing**: positive vendored registry/git import scaffolds, negative stale checksum/missing vendor/ambient-cache cases, generated Nickel shape checks, and an offline Cargo build smoke using the generated vendor input.

## Out of Scope

- Running `cargo vendor` or fetching dependencies during import.
- Claiming full Cargo compatibility or Cargo-free native Rust execution.
- Supporting arbitrary Cargo source replacement backends beyond explicit local vendored directory sources in this change.
- Replacing the native `rust-plan` verification lane.
