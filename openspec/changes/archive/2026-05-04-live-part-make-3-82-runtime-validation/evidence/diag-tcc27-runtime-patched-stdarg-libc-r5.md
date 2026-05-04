# TinyCC 0.9.27 patched stdarg/Mes libc diagnostic (r5)

Task-ID: live-part-make-3-82-runtime-validation V3 blocker diagnosis
Covers: remaining `rc=21` after the static REX/GOT repair and `diag-tcc27-runtime-format-inspect-r1`.

## Command

```sh
nix shell nixpkgs#bubblewrap -c env \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  ~/.cargo-target/debug/crunch --json \
  --store "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store" \
  --state-dir "$PWD/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-state" \
  bootstrap validate bootstrap/diag-tcc27-runtime-patched-stdarg-libc.ncl \
  --warmup bootstrap/tinycc.ncl \
  --resume \
  --evidence-dir target/bootstrap-validation/diag-tcc27-runtime-patched-stdarg-libc-r5
```

## Result

The validation runner passed. The diagnostic rebuilt Mes `libc.a` from Mes source using TinyCC 0.9.26, but with a TinyCC-compatible x86_64 SysV `stdarg.h` and a local `va_list.c` helper, then linked the same two-object TinyCC 0.9.27 static runtime probe against that patched libc.

Runtime result: `rc=0`.

Stdout:

```text
diag-tcc27-runtime-patched-stdarg-libc
score=43
value=42

```

Stderr:

```text

```

## Interpretation

This positively identifies the previous `rc=21` mismatch as Mes libc's x86_64 varargs ABI, not the already-repaired static ELF startup/GOT path. The successful combination was:

1. use TinyCC's x86_64 `va_list` layout/macros for Mes objects compiled by TinyCC;
2. provide `__va_start`/`__va_arg` support in the libc archive;
3. compile the rebuilt Mes libc with the predecessor TinyCC 0.9.26, not the current TinyCC 0.9.27 binary (the 0.9.27 compiler still segfaulted while compiling unified Mes libc in r2-r4).

The next implementation slice should port this into `bootstrap/tinycc.ncl`: rebuild/carry a patched Mes `libc.a` with the predecessor compiler for the installed TinyCC 0.9.27 runtime, then rerun `bootstrap/diag-tcc27-static-runtime-inspect.ncl` and finally return to `bootstrap/make-tcc.ncl`.

Evidence files in this directory use prefix `diag-tcc27-runtime-patched-stdarg-libc-r5`.

Output path:

```text
/home/brittonr/git/crunch/crunch/.crunch-drain/diag-tcc27-static-runtime-inspect-warmup-r4-store/vgkziyzbz6l1s3mhn9zs68g1rv96qnmr-diag-tcc27-runtime-patched-stdarg-libc
```
