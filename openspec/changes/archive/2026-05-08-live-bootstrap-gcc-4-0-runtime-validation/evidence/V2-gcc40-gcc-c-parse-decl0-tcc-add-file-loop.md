# V2 gcc40 c-parse decl0 tcc add-file loop diagnostic

Task-ID: V2 focused TinyCC `tcc.c`/`libtcc.c` file-add loop diagnostic for the GCC 4.0 c-parse `decl0` predecessor compile boundary.
Covers: bootstrap.gcc40.runtime-validation

## Commands

The Crunch diagnostic derivation was extended in `bootstrap/diag-gcc40-c-parse-boundary.ncl` with source-level markers around:

- `tcc_open()` entry,
- `tcc_add_file_internal()` entry,
- public `tcc_add_file()` entry,
- the `tcc.c` main loop immediately before `tcc_add_file(s, f->name)`.

A direct local replay was run from restored prior Crunch outputs because the fresh derivation attempt still stopped in a cold/stale predecessor dependency before reaching the diagnostic root:

```sh
timeout 590 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --json \
  --store "$PWD/.crunch-drain/store" --no-substitute \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
# failed before target diagnostic with: dependency bash-2.05b-tcc.drv failed
# saved root log: /home/brittonr/.local/state/crunch/logs/0q1068nl6v7gr7l96s3rgvlmn0pimlv3-diag-gcc40-c-parse-boundary.drv.log

openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-add-file-loop-20260510/run-local-probes.sh \
  > openspec/changes/archive/2026-05-08-live-bootstrap-gcc-4-0-runtime-validation/evidence/gcc40-cparse-decl0-add-file-loop-20260510/focused-local-probes.txt 2>&1
```

Replay inputs:

- TinyCC: `.crunch-drain/post621-restored-store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2/bin/tcc`
- musl headers/libs: `.crunch-drain/post621-restored-store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl`
- TinyCC source: `.crunch-drain/post621-restored-store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src`

## Focused results

| Phase | Result |
| --- | --- |
| `compile-only-quiet` | `rc=139`, no object emitted |
| `compile-only-verbose` | `rc=139`, stdout reached `tcc version 0.9.27` and `-> -> %s` |
| `link-object-default` | `rc=1`, because `/tmp/tcc-add-file-instrumented.o` was missing after compile-only crash |
| `link-object-explicit-archives` | `rc=1`, same missing-object boundary even with explicit `libtcc1.a`/`libc.a` |

Input existence after the failed compile:

- `tcc.c`: present
- `/tmp/tcc-add-file-instrumented.o`: missing
- `libtcc1.a`, `libc.a`, `crt1.o`, `crti.o`, `crtn.o`: present

## Interpretation

The verbose predecessor reaches its existing file-add print before the crash, but the source string is corrupted as `-> %s`; no source-level `diag-tcc-file:` markers can execute because the instrumented compiler object is never produced. The next repair should therefore not chase linker inputs or CRT archive availability. The live blocker remains in the predecessor compiler while processing the one-source `tcc.c` input before object emission, with the add-file/name formatting path now the narrowest visible boundary.
