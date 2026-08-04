# Design: Cargo profile codegen flags

## Context

`src/rust_plan.rs` carries a profile string through units and receipts but
never turns it into rustc codegen flags. Cargo's documented built-in tables
are the compatibility target:

| setting | dev | release |
|---|---|---|
| opt-level | 0 | 3 |
| debug | full (2) | none (0) |
| debug-assertions | true | false |
| overflow-checks | true | false |
| lto | false | false |
| panic | unwind | unwind |
| incremental | true | false |
| codegen-units | 256 | 16 |
| rpath | false | false |

`test` inherits `dev`. `bench` inherits `release`.

## Pure core

Add pure functions with no I/O:

- `resolve_builtin_profile(name) -> Result<ProfileSettings, ProfileError>`
  returns the full resolved setting table for `dev`, `release`, `test`, and
  `bench`. Unknown names fail closed.
- `profile_codegen_args(settings) -> Vec<(String, String)>` returns ordered
  `-C` option pairs: `opt-level`, `debuginfo`, `debug-assertions`,
  `overflow-checks`, `codegen-units`.
- `profile_metadata_material(settings) -> String` returns the stable string
  that enters the metadata disambiguator.

Assertions: resolved `dev` and `release` differ in every core setting;
resolved `test` equals `dev`; resolved `bench` equals `release`;
`profile_codegen_args` output is deterministic for identical input.

## Shell wiring

- `native_rustc_args`, the Cargo-derived unit argument builder, the
  integration-test argument builder, and the dev-dependency lib argument
  builder append `profile_codegen_args` output with the existing
  `RUSTC_CODEGEN_OPTION_FLAG`.
- `rustc_unit_metadata_disambiguator` adds the profile material so units that
  differ only by profile get distinct `-C metadata` hashes.
- Debug setting maps `false`/`0` to `-C debuginfo=0` and `true`/`2` to
  `-C debuginfo=2`. String forms (`limited`, `full`, `line-tables-only`) are
  out of scope for this change and fail closed if supplied later.
- `-C debug-assertions` controls `cfg(debug_assertions)`. Mantle MUST NOT
  rely on rustc's implicit opt-level-0 behavior.

## Determinism policy

- Mantle disables incremental compilation for all profiles. Receipts record
  `incremental = false` as an explicit Mantle deviation from Cargo's `dev`
  default.
- Mantle uses the Cargo default `codegen-units` values (256 dev, 16 release).
  Fixed-point proofs that need one codegen unit bind the override into the
  receipt instead of hiding it.

## Non-claims

- This change does not prove Cargo behavior for every profile setting. `lto`,
  `panic`, `rpath`, `strip`, and `split-debuginfo` remain at rustc defaults
  and are recorded as unsupported-profile-surface blockers when a manifest or
  CLI input requests them.
- Profile equality of flags is not a proof that produced binaries match Cargo
  byte for byte.
