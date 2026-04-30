## Why

`repair-make-tcc-amd64-varargs` proved that the first GNU make pass cannot be repaired locally while the predecessor compiler boundary is unstable. The archived `bootstrap/tinycc.ncl` output reports `tcc version 0.9.27 (x86_64 Linux)`, but it hangs compiling even a trivial C file. Falling back to `tinycc 0.9.26` builds make objects, but the resulting GNU make still corrupts amd64 runtime state and exits without executing simple recipes.

The make repair therefore needs a predecessor compiler repair: `bootstrap/tinycc.ncl` must produce a TinyCC 0.9.27 that can compile trivial C on amd64, not only print a version string.

## What Changes

- Repair the Mes-linked TinyCC 0.9.27 amd64 compile path in `bootstrap/tinycc.ncl` or adjust its Mes runtime handoff so `tcc -c` terminates successfully.
- Preserve the already-validated live-bootstrap source patches and output contract for `tinycc 0.9.27`.
- Add smoke evidence that the compiler can compile a trivial object and fails malformed input cleanly.

## Scope

- **In scope**: `bootstrap/tinycc.ncl`, TinyCC 0.9.27 source patches, Mes runtime compatibility needed for `tcc -c`, and direct compiler smoke evidence.
- **Out of scope**: building GNU make, musl, or downstream autotools stages; those stay in their own changes.

## Evidence Needed

Completion requires a source-pin audit, successful `crunch build bootstrap/tinycc.ncl`, version smoke, `tcc -c hello.c` object smoke, malformed-input negative smoke with non-signal failure, and host-leakage scan.

## Parent

Discovered by `openspec/changes/repair-make-tcc-amd64-varargs` task I2.
