# Store prefix integration fix

## Why

The `--store-prefix` CLI flag (added in the configurable store prefix
change) had no effect on sandbox builds. The configured prefix reached
`ConversionCache` for output path computation but never reached `Builder`,
which hardcoded `/nix/store` in two places. Builds with the default
`/crunch/store` prefix failed at runtime: the sandbox mounted inputs at
`/nix/store` while `$out` pointed to `/crunch/store`, so output files
couldn't be found after the build completed.

A secondary issue: smoke tests used `mkdir`/`chmod`/`ln` inside the
sandbox without qualifying them via busybox. The nixpkgs busybox-static
binary lacks `CONFIG_FEATURE_SH_STANDALONE`, so external applets aren't
auto-discovered when `PATH=/path-not-set`.

## What Changes

- **Builder constructors accept `store_dir`**: `Builder::new()` and
  `Builder::with_state_dir()` gain a `store_dir: &str` parameter.
  Internally use `StoreHandle::from_services_with_store_dir()` instead
  of `from_services()`.

- **BuildRequest uses prefix-aware output paths**: `build_request.rs`
  changed from `Output::path_str()` (hardcoded `/nix/store`) to
  `Output::path_str_with_prefix(store_dir)`.

- **Smoke tests use busybox prefix**: Build scripts that need `mkdir`,
  `chmod`, `ln` use `BB=/bin/busybox; $BB mkdir` pattern (same as
  bootstrap scripts).

- **Integration tests use `--store <tempdir>`**: Avoids dependency on
  writable `/nix/store` and works with any prefix.

## Capabilities

### Modified Capabilities
- `Builder::new`: additional `store_dir: &str` parameter
- `Builder::with_state_dir`: additional `store_dir: &str` parameter
- `derivation_to_build_request`: output paths use configured prefix

## Impact

- **Files**: `orchestrate.rs` (50+ call sites), `build_request.rs` (1 line),
  `worker.rs` (17 call sites), `main.rs` (2), `bootstrap.rs` (1),
  `tests/integration.rs` (2 tests), `tests/integration_build.rs` (5),
  `tests/smoke.rs` (3 build scripts)
- **APIs**: `Builder::new` and `Builder::with_state_dir` gain a parameter.
  Callers must pass `store_dir`. Existing test callers pass
  `nix_compat::store_path::STORE_DIR`.
- **Breaking**: None for CLI users. Library callers of `Builder::new` /
  `Builder::with_state_dir` must add the `store_dir` argument.
