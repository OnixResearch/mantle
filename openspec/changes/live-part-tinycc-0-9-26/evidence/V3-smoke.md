Task-ID: V3
Covers: bootstrap.part.tinycc.0.9.26,bootstrap.part.tinycc.0.9.26.selfcompile

# Tinycc 0.9.26 output-contract smoke

Result: PASS.

The archived blocker-fix smoke checked the produced output under a bwrap binding from the physical custom store path to the logical `/crunch/store/...-tinycc-0.9.26` path embedded by the compiler.

Checked output files:

```text
bin/tcc
bin/tcc-0.9.26
lib/mes/libc.a
lib/mes/tcc/libtcc1.a
```

Smoke command actions:

- run `bin/tcc -version`;
- run `bin/tcc-0.9.26 -version`;
- compile and run a trivial static C program with `bin/tcc`;
- compile and run the same program with `bin/tcc-0.9.26`.

Transcript:

```text
tcc version 0.9.26 (x86_64 Linux)
tcc version 0.9.26 (x86_64 Linux)
V3-smoke-pass
```

This is direct produced-compiler evidence. `tcc-mes -version` alone is not used as the acceptance boundary.

Primary evidence: `openspec/changes/archive/2026-04-29-fix-tinycc-mes-bufferedfile-codegen/evidence/V3-smoke.md`.
