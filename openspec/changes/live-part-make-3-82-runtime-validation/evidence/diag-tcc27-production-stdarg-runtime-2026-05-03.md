# Production TinyCC stdarg/runtime repair evidence

Task-ID: V3
Covers: r[bootstrap.part.make.3.82.runtime-validation]
Captured: 2026-05-03T20:25:32Z

## What changed

`bootstrap/tinycc.ncl` now rebuilds the Mes 0.27.1 x86_64 libc used by TinyCC 0.9.27 with a TinyCC-compatible `stdarg.h` ABI and a companion `va_list.o` implementation. The derivation installs `va_list.o`, rebuilds `libc.a` from a unified Mes libc source, links the final TinyCC with `va_list.o`, and verifies the added runtime object is present.

This ports the earlier focused `diag-tcc27-runtime-patched-stdarg-libc` proof into the production TinyCC derivation.

## Verification commands

```sh
./target/debug/crunch eval bootstrap/tinycc.ncl >/tmp/tinycc-eval4.out 2>/tmp/tinycc-eval4.err
nix shell nixpkgs#clang -c env CARGO_TARGET_DIR=target/source-pin-check \
  cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc.ncl \
  > /tmp/tinycc-source-pins.log 2>&1
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ~/.cargo-target/debug/crunch --json \
  --store "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store" \
  --state-dir "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state" \
  bootstrap validate bootstrap/tinycc.ncl \
  --evidence-dir target/bootstrap-validation/tinycc-stdarg-runtime-r3 \
  --resume
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ~/.cargo-target/debug/crunch --json \
  --store "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store" \
  --state-dir "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state" \
  bootstrap validate bootstrap/diag-tcc27-static-runtime-inspect.ncl \
  --warmup bootstrap/tinycc.ncl \
  --evidence-dir target/bootstrap-validation/diag-tcc27-static-runtime-stdarg-r1 \
  --resume
.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store/hx941nj7yrr4qbl915rfr6frvklcxmzy-diag-tcc27-static-runtime-inspect/bin/diag-tcc27-static-runtime
.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store/8kc0mzcs5hj1f01nwckbzsk8l3nbfhvw-tinycc-0.9.27/bin/tcc -v
```

## Results

- `crunch eval bootstrap/tinycc.ncl`: exit 0.
- Source-pin audit: exit 0, `source-pin audit: 1 files, 2 fetch blocks, 0 issues`.
- `bootstrap/tinycc.ncl` validation: status `Passed`, build exit `Some(0)`, no coarse host-path leakage findings.
- `bootstrap/diag-tcc27-static-runtime-inspect.ncl` validation with TinyCC warmup: status `Passed`, build exit `Some(0)`, no coarse host-path leakage findings.
- Host execution of the produced static diagnostic binary: exit 0 with stdout `diag-tcc27-static-runtime`, `score=43`, `value=42`.
- Produced TinyCC reports `tcc version 0.9.27 (x86_64 Linux)`.

## Transcript locations

Scratch transcripts (not committed):

- `target/bootstrap-validation/tinycc-stdarg-runtime-r3/validation-summary.md`
- `target/bootstrap-validation/tinycc-stdarg-runtime-r3/build.stdout.log`
- `target/bootstrap-validation/diag-tcc27-static-runtime-stdarg-r1/validation-summary.md`
- `target/bootstrap-validation/diag-tcc27-static-runtime-stdarg-r1/build.stdout.log`
- `.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state/logs/a1qpiab5nq53vjfzvifg464hngs36gcr-tinycc-0.9.27.drv.log`
- `.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state/logs/d7ly56nw8zadp6nf2jk8w3bj9rksyfr3-diag-tcc27-static-runtime-inspect.drv.log`

## Full Make validation status

A follow-up `bootstrap validate bootstrap/make-tcc.ncl --warmup bootstrap/tinycc.ncl` attempt using fresh `.crunch-drain/make-tcc-stdarg-runtime-r2-*` scratch state was stopped after roughly 10 minutes while still rebuilding the Mes warmup prerequisite and before producing Make build evidence. V3 remains open until `bootstrap/make-tcc.ncl` itself builds and both `make --version` and a simple Makefile smoke pass.
