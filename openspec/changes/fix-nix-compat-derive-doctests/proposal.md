## Why

17 doctests in `vendor/nix-compat-derive/src/lib.rs` fail with:

```
error[E0433]: cannot find `nix_compat` in the crate root
```

The derive macros (`NixDeserialize`, `NixSerialize`) expand to code that
references `::nix_compat::...` — the proc macro assumes it's used inside
a crate that has `nix_compat` as a dependency. Doctests compile as
standalone crates with only the proc macro crate in scope, so
`::nix_compat` doesn't resolve.

This is a known pattern with proc macro crates. The doctests worked in the
original snix/tvix repo because they were run as part of a workspace where
`nix-compat` was always available. In crunch's vendored layout, the
doctests are compiled in isolation.

The test run shows `17 failed` every time, which trains you to ignore test
output. Broken windows.

## What Changes

Fix the 17 doctests so `cargo test --workspace` passes clean (or explicitly
exclude/skip them if the fix is non-trivial).

## Capabilities

### Modified Capabilities
- `test-suite-clean`: `cargo test --workspace` runs without failures.

## Impact

- **Files**: `vendor/nix-compat-derive/src/lib.rs` (doctest annotations)
- **APIs**: None
- **Dependencies**: None
- **Testing**: `cargo test -p nix-compat-derive --doc`
