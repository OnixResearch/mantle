# TinyCC 0.9.27 runtime format/varargs diagnostic (format-inspect-r1)

Task-ID: live-part-make-3-82-runtime-validation diagnostic follow-up
Covers: remaining `rc=21` blocker after `diag-tcc27-static-runtime-rexgot-r2`.

## Command

```sh
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ~/.cargo-target/debug/crunch --json \
  --store "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store" \
  --state-dir "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state" \
  bootstrap validate bootstrap/diag-tcc27-runtime-format-inspect.ncl \
  --warmup bootstrap/tinycc.ncl \
  --resume \
  --evidence-dir target/bootstrap-validation/diag-tcc27-runtime-format-inspect-r1
```

## Result

The diagnostic build and link passed, and the generated static executable was run inside the builder with its rc/stdout/stderr preserved under `$out/src`.

Runtime result: `rc=139`.

Captured stdout begins:

```text
diag-tcc27-runtime-format-inspect
v=6336458
loop i=6336529 word=
```

Captured stderr:

```text
Segmentation fault (core dumped)

```

## Interpretation

This narrows the post-GOT blocker to x86_64 varargs/runtime formatting. Adding diagnostic `printf` calls makes the static binary segfault after printing corrupted vararg values such as `v=6336458` instead of `v=42` and `loop i=6336529`. The prior rexgot diagnostic already proved static startup, function pointer relocation, and PLT-backed GOT materialization now progress; this run shows Mes/TinyCC varargs consumers still read arguments incorrectly.

Likely seam: Mes libc's `include/mes/stdarg.h` / varargs ABI is stack-pointer based, while TinyCC-generated x86_64 callers pass varargs in registers according to the x86_64 ABI. Repair should target the runtime varargs ABI or rebuilt varargs-sensitive libc objects, not more PLT/GOT startup work.

Evidence files in this directory use prefix `diag-tcc27-runtime-format-inspect-r1`.

Output path:

```text
/home/brittonr/git/crunch/crunch/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store/zb27qy9zmz493h4650f7p8l6lb35xz28-diag-tcc27-runtime-format-inspect
```
