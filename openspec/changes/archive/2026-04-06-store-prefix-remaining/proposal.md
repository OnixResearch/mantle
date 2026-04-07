# Store prefix remaining issues

## Why

Integration testing of the configurable store prefix revealed three
issues that the store-prefix-integration-fix didn't address. These
block the `self-contained-bootstrap` openspec from completion.

### 1. Bootstrap .ncl files hardcode `/nix/store` in shell globs

All 10 bootstrap .ncl files use `for d in /nix/store/*-gcc` to find
tools inside the sandbox. With `--store-prefix /crunch/store`, inputs
mount at `/crunch/store`, so these globs match nothing. 56 occurrences
across bootstrap/*.ncl and `self_build.rs::generate_self_build_ncl()`.

### 2. Smoke test isolation

Parallel smoke tests share `~/.local/state/crunch/pathinfo.redb`. When
one test holds the db lock, others fall back to in-memory storage and
lose cache state between runs. `smoke_build_cached_on_second_run` fails
intermittently.

### 3. `smoke_build_multi_derivation_file` eval failure

The multi-derivation smoke test passes a Nickel array `[drv1, drv2]` to
`crunch build`. The eval layer rejects this with "expected a Derivation
record or a record of Derivations". Either the eval layer needs to
accept arrays or the test needs updating to use the supported record
syntax.

## What Changes

- **Bootstrap .ncl: use `$NIX_STORE` env var** instead of hardcoded
  `/nix/store` in all shell globs. The sandbox sets `NIX_STORE` to the
  configured prefix. `for d in $NIX_STORE/*-gcc` works with any prefix.

- **`generate_self_build_ncl()`: same fix** — replace `/nix/store` globs
  with `$NIX_STORE`.

- **Smoke test isolation**: each smoke test sets `CRUNCH_STATE_DIR` to a
  temp directory (or the binary gets a `--state-dir` flag) so parallel
  tests don't contend on the shared pathinfo.redb.

- **Multi-derivation eval**: either support arrays in the eval entry
  point or change the test to use a record `{ alpha = drv1, beta = drv2 }`.

## Capabilities

### Modified Capabilities
- `bootstrap/*.ncl`: portable across store prefixes
- `generate_self_build_ncl()`: portable across store prefixes
- Smoke tests: isolated state, no shared db lock

## Impact

- **Files**: 10 bootstrap .ncl files (56 glob sites), `self_build.rs`,
  `tests/smoke.rs`, possibly `crunch-eval` or `main.rs` for
  multi-derivation support
- **Risk**: low — shell glob replacement is mechanical. The `$NIX_STORE`
  env var is already set correctly in the sandbox.
