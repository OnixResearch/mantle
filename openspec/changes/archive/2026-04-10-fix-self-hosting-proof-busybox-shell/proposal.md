## Why

The self-hosting proof change is complete on paper, but the proof is not
archiveable yet. `./scripts/prove-self-hosting.sh --check` passes on this
host, while the full proof still fails in stage0 before it can produce a
stage1 binary.

The failing root build is `busybox.drv`. The stage0 bootstrap reaches the
crunch-built `bwrap` root, then the busybox sandbox dies with:

```text
bwrap: execvp /bin/sh: No such file or directory
```

That means proof-related OpenSpec changes cannot be honestly archived from the
preflight alone.

## What We Know

- Reproduced on 2026-04-10 with `./scripts/prove-self-hosting.sh`
- `./scripts/prove-self-hosting.sh --check` passed first, so the host had the
  expected nightly/clang/pkg-config/openssl/bwrap prerequisites
- The failing test was `self_hosting_stage0_stage1_stage2`
- Stage0 got through `bwrap.ncl` and started `busybox.ncl`
- Final failure excerpt:
  - `FAILED: /nix/store/rfwv7df56njd40hkx3knm8gr7b7pm7v1-busybox.drv`
  - `store error: build: nonzero exit code: exit status: 1`
  - `bwrap: execvp /bin/sh: No such file or directory`
- Evidence bundle from the failing run:
  - audit bundle: `target/test-audit/self-hosting/stage0-1415381-1775845586004697542`
  - diagnostics: `/tmp/.tmpBOw2nz/stage0-diagnostics.txt`
  - stderr capture: `/tmp/.tmpBOw2nz/stage0-stderr.txt`
- This is separate from the source-binding / exact-bootstrap-input proof work.
  The proof-input change validated, but the underlying proof still fails.

## What Changes

- Trace why the busybox stage still ends up execing `/bin/sh` inside bwrap
  instead of a usable static shell
- Check whether the failure comes from:
  - generated bootstrap derivation content
  - `vendor/snix-build` sandbox shell selection
  - stage0 environment plumbing vs compile-time default fallback
- Add a regression test or proof assertion that fails before archive if the
  busybox bootstrap path falls back to unusable `/bin/sh`
- Re-run the full self-hosting proof after the fix, not just `--check`

## Scope

- **In scope**: stage0 proof failure in the busybox bootstrap sandbox shell path
- **Out of scope**: byte-for-byte stage1 == stage2 fixed-point claims, general
  attestation/provenance work, and the already-completed staged-source/bootstrap-input
  binding change

## Evidence

```text
$ ./scripts/prove-self-hosting.sh --check
self-hosting proof check passed
...
proof command: cargo test -p crunch --test self_hosting -- --ignored --nocapture

$ ./scripts/prove-self-hosting.sh
...
FAILED: /nix/store/rfwv7df56njd40hkx3knm8gr7b7pm7v1-busybox.drv
  store error: build: nonzero exit code: exit status: 1
bwrap: execvp /bin/sh: No such file or directory
...
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 846.89s
```
