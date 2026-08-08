# Design: Cargo profile manifest and selection model

## Context

Cargo's documented model:

1. Only the workspace root manifest `[profile]` table is read. Dependency
   manifests are ignored.
2. Custom profiles must set `inherits`. The base is a built-in profile or
   another custom profile.
3. Overrides resolve first-match-wins:
   `[profile.<p>.package.<name>]`, then `[profile.<p>.package."*"]`, then
   `[profile.<p>.build-override]`, then `[profile.<p>]`, then built-in
   defaults.
4. Overrides cannot set `panic`, `lto`, or `rpath`.
5. Selection: build/check/run/rustc default to `dev`, test to `test`, bench
   to `bench`, install to `release`. `--release` equals `--profile release`.

## Pure core

Add pure functions over parsed manifest data:

- `parse_profile_table(root_manifest) -> Result<ProfileTable, ProfileError>`
  reads only the root manifest. Unknown setting keys fail closed. Forbidden
  keys in overrides fail closed.
- `resolve_profile(table, name) -> Result<ProfileSettings, ProfileError>`
  walks `inherits` with a bounded depth limit and rejects cycles.
- `select_unit_profile(table, selection, unit_role, package_name,
  is_workspace_member) -> ProfileSettings` implements the precedence ladder.
- `select_command_profile(command, release_flag, profile_flag) -> String`
  implements the command default table.

Assertions: a custom profile with one unset field inherits that field from
its base; `"*"` never matches workspace members; a named-package override
beats `"*"`; `build-override` beats the profile only for host units.

## Shell wiring

- `src/rust_plan.rs` parses the root manifest table during planning and
  passes the resolved settings to the codegen flag lowering from
  `adopt-cargo-profile-codegen-flags`.
- `src/cargo_import.rs` replaces the hard `debug|release` blocker with
  resolution through the same table. Unsupported resolved settings still fail
  closed with the existing blocker class.
- Version-qualified package specs (`foo:2.1.0`) are rejected with an
  unsupported-override-spec blocker in this change. Name-only specs are
  supported first.

## Recorded caveats

- Overrides that change `opt-level` interact with generic monomorphization:
  at opt-level 2 or 3 rustc does not share monomorphized generics across
  crates. Receipts record per-unit opt-level so this interaction stays
  diagnosable. This is a documented Cargo caveat, not a Mantle defect.
- Config-file and environment profile overrides (Cargo `config.toml` and
  `CARGO_*` variables) stay out of scope. Mantle owns an explicit CLI and
  manifest surface only.
