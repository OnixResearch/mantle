# V2 GCC 4.0 c-parse auto-host 80/90 boundary

Task-ID: V2
Covers: bootstrap.gcc40.runtime-validation

## Summary

A focused Crunch diagnostic build of `bootstrap/diag-gcc40-c-parse-boundary.ncl` narrowed the remaining `auto-host.h` / `system.h` TinyCC segmentation boundary after the verified `ansidecl.h` inline repair.

The previous recorded boundary was broad: first 50 generated `auto-host.h` `#define` entries followed by `system.h` compiled, while first 100 segfaulted. This slice added intermediate sandbox-generated prefix probes and reran the real diagnostic. The new boundary is:

- first 80 generated `auto-host.h` defines + `system.h`: `rc=0`
- first 90 generated `auto-host.h` defines + `system.h`: `rc=139`

The diagnostic still exits `1` overall, as expected, because it intentionally reproduces the remaining `gcc/c-parse.c` TinyCC segmentation fault after recording the matrix.

## Command

```sh
nix shell nixpkgs#bubblewrap nixpkgs#clang -c bash -lc '
  export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
  export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
  export SNIX_BUILD_SANDBOX_SHELL=/bin/sh CC=clang CARGO_TARGET_DIR=target
  target/debug/crunch build --no-substitute \
    --store "$PWD/.crunch-drain/gcc40-autohost-bisect-bg/store" \
    --state-dir "$PWD/.crunch-drain/gcc40-autohost-bisect-bg/state" \
    bootstrap/diag-gcc40-c-parse-boundary.ncl
'
```

Exit status: `1` (expected diagnostic failure after matrix capture).

Focused transcript: `V2-gcc40-gcc-c-parse-autohost-80-90-boundary-build.diag.log`.

## Boundary excerpt

```text
diag-cparse: cparse_inc_config rc=0
diag-cparse: autohost_defines_10_system rc=0
diag-cparse: autohost_defines_25_system rc=0
diag-cparse: autohost_defines_50_system rc=0
diag-cparse: autohost_defines_60_system rc=0
diag-cparse: autohost_defines_70_system rc=0
diag-cparse: autohost_defines_80_system rc=0
diag-cparse: autohost_defines_90_system rc=139
diag-cparse: autohost_defines_95_system rc=139
diag-cparse: autohost_defines_100_system rc=139
diag-cparse: autohost_defines_200_system rc=139
diag-cparse: autohost_defines_400_system rc=139
diag-cparse: cparse_manual_config_system rc=139
diag-cparse: cparse_autohost_system rc=139
diag-cparse: cparse_ansidecl_system rc=0
diag-cparse: cparse_inc_system_only rc=0
diag-cparse: cparse_inc_system rc=139
diag-cparse: cparse_full rc=139
```

## Interpretation

The `ansidecl.h` inline-empty boundary remains repaired (`cparse_ansidecl_system rc=0`). The next unresolved frontier is the generated `auto-host.h` macro group introduced after the first 80 `#define` entries and before the first 90 entries when followed by `system.h` under the real GCC 4.0 c-parse compile flags.
