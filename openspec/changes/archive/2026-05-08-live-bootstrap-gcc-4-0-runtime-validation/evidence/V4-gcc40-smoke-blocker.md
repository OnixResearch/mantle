# V4 GCC 4.0 C/C++ smoke blocker

Task-ID: V4
Covers: bootstrap.gcc40.runtime-validation
Captured: 2026-05-08T20:53:38Z

## Scope

The C and C++ compiler smoke tests are conditional on a produced `gcc-4.0.4` output. The current validation evidence does not produce that output: the build boundary remains inside the predecessor TinyCC/Mes path before direct `decl0` runtime markers or a GCC output can exist.

## Evidence reviewed

- `evidence/V2-gcc40-gcc-c-parse-decl0-libtcc-compile-signature.md`
- `evidence/V2-gcc40-gcc-c-parse-decl0-libtcc-vastart-end-full.md`
- `evidence/V2-gcc40-gcc-c-parse-decl0-libtcc-vastart-end-prefix-growth.md`
- `evidence/V2-gcc40-gcc-c-parse-decl0-libtcc-vastart-end-prefix-extended.md`
- `evidence/V2-gcc40-gcc-c-parse-decl0-libtcc-compile-shape.md`
- `evidence/V3-gcc40-c-parse-host-leakage-scan.md`

## Result

No C/C++ smoke executable was run because there is no `gcc-4.0.4` output to execute. The terminal deterministic blocker recorded by the current evidence is:

- `libtcc.c` still returns `rc=139` in the predecessor compiler path before direct `decl0` runtime markers can execute.
- The latest signature/name matrix keeps `libtcc_compile_signature_*` variants at `rc=139` under both common and `ONE_SOURCE=1` flags.
- `V3` proves the captured failed boundary transcript has no host compiler/libc/shell/Nix/legacy-provider leakage; it does not prove GCC builds.

This closes V4 as a negative smoke-gate result for this validation drain: smoke proof is blocked by the recorded build boundary, not by missing test commands or host fallback.
