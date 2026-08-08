# Design: Cargo profile build overrides

## Context

Cargo documents one built-in build-override table that applies to build
scripts, proc macros, and their dependencies:

```toml
[profile.dev.build-override]
opt-level = 0
codegen-units = 256
debug = false # when possible

[profile.release.build-override]
opt-level = 0
codegen-units = 256
```

Mantle currently derives build-script env directly from the active profile
name (`release`/`bench` gives `OPT_LEVEL=3`, `dev` gives `DEBUG=true`).

## Pure core

Replace the profile-name branch with a pure resolver:

- `build_override_env(profile: &str, dual_use: bool) -> BTreeMap<String, String>`
- `OPT_LEVEL` is `0` for every profile.
- `DEBUG` is `false`, except when `dual_use` is true. Then the value equals
  the target profile's debug setting (`true` for dev-like profiles, `false`
  for release-like profiles).
- `NUM_JOBS` stays `1` (Mantle deterministic constant, already correct).

Assertions: release build-override env differs from the release target env;
dev build-override env differs from the dev target env on `DEBUG`; identical
inputs produce identical maps.

## Shell wiring

- `append_build_script_profile_env` calls the resolver with
  `dual_use = false` for host-only build scripts and proc macros.
- Host-dependency unit planning marks a package `dual_use` only when the same
  package is also selected as a normal target dependency in the current unit
  graph.
- Cargo-derived and native topology paths share the resolver so the two
  planning modes do not drift.

## Non-claims

- Explicit `[profile.*.build-override]` values from manifests are owned by
  the `adopt-cargo-profile-manifest-config` change. This change fixes only
  the built-in defaults.
- Cargo may build a dual-use dependency twice when an explicit build-override
  differs from the target profile. Mantle builds it once under the recorded
  dual-use value and records that choice in the receipt.
