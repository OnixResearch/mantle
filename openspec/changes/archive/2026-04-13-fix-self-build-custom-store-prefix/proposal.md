## Why

Actual `crunch self-build` validation against the current checkout still fails
under the default logical store prefix `/crunch/store`, even after the stage0
source-staging rewrite and the CA-mapping cache fix.

Observed during explicit end-to-end runs:

- task 205 first reached stage `[3/4] Building crunch...` and failed with a
  Nickel contract error:
  `input string must be a valid store path, got: /crunch/store/...-crunch-src`
- after broadening `lib/contracts.ncl` and rebuilding, task 210 reached stage
  `[3/4] Building crunch...` and then failed immediately with:
  `crunch: invalid store path: /crunch/store/w2b8lnrhm2qy3jafza52d2n0mp6iv9cx-busybox`

That means the stage0 path is now exercised for real, but the self-build path
still contains `/nix/store` assumptions deeper in the stack.

## What We Know

- `src/self_build.rs` now stages sources without host `git`/`tar`/`cp` glue.
- `crates/crunch-store/src/handle.rs` needed one custom-prefix fix already:
  CA mapping lookups now parse with `StorePath::from_absolute_path_with_prefix`.
- `lib/contracts.ncl` previously hardcoded `/nix/store` in `StorePath` and
  `Input`; that blocked stage3 evaluation for `/crunch/store/...` inputs.
- Even after fixing those Nickel contracts, the rebuilt binary still rejects a
  generated `/crunch/store/...-busybox` path during stage3, so at least one
  Rust-side parser or validator still assumes `/nix/store`.
- The failing evidence came from real `crunch self-build` runs, not unit-only
  tests.

## What Changes

- Audit the stage3 self-build path for any remaining `StorePath::from_absolute_path`
  calls or equivalent logic that should use the configured store prefix.
- Add regression coverage that runs the self-build path far enough to validate
  `/crunch/store/...` source and bootstrap-tool inputs under the default prefix.
- Re-run an end-to-end `crunch self-build` after the parser fixes land.

## Scope

- **In scope**: self-build/store-path validation bugs that break the default
  `/crunch/store` stage3 path.
- **Out of scope**: the stronger non-Nix-host proof path tracked by
  `non-nix-stage0-bootstrap`, and unrelated busybox/kernel-header warnings.

## Evidence

From task 205 / task 210:

```text
[3/4] Building crunch...
error: evaluation failed
Nickel evaluation error:
error: contract broken by the value of `inputs`
input string must be a valid store path, got: /crunch/store/...-crunch-src
```

```text
[3/4] Building crunch...
error: build failed
crunch: invalid store path: /crunch/store/w2b8lnrhm2qy3jafza52d2n0mp6iv9cx-busybox
```
