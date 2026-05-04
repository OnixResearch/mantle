# I3 source-level hardening

Task-ID: I3
Covers: `bootstrap.part.bzip2.1.0.8.tcc`

## Changes

Updated `bootstrap/bzip2-tcc.ncl` to make the source-level contract explicit before runtime validation:

- added `provenance: fosslinux/live-bootstrap steps/bzip2-1.0.8/sources` beside the source pin;
- added `first-consumer: bootstrap/bzip2-tcc.ncl` beside the source pin;
- documented the upstream `bzip2 --help` smoke and Crunch's expanded installed output contract;
- fail-closed on `bzip2`, `bunzip2`, `bzcat`, and `bzip2recover` installed paths;
- added installed command help/usage probes for the four output commands;
- avoided the unused `libbz2.a` archive because the Mes-built TinyCC `tcc -ar` path segfaulted during this part, while the part installs only executables;
- supplied a local `utime.h` shim and disabled metadata-preservation calls not provided by the Mes libc handoff (`utime`, `fchmod`, `fchown`);
- chmodded linked executables before the in-derivation smoke probes.

## Verification

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/bzip2-tcc.ncl \
  > openspec/changes/archive/2026-05-04-live-part-bzip2-1-0-8-tcc/evidence/V1-source-pin-audit.log 2>&1
```

Exit status: 0

Transcript: `openspec/changes/archive/2026-05-04-live-part-bzip2-1-0-8-tcc/evidence/V1-source-pin-audit.log`.

Key output:

```text
source-pin audit: 1 files, 1 fetch blocks, 0 issues
```
