# I3 source-level hardening

Task-ID: I3
Covers: `bootstrap.part.bzip2.1.0.8.musl`

## Changes

Updated `bootstrap/bzip2-1.0.8-musl.ncl` to make the source-level contract explicit before runtime validation:

- added `provenance: fosslinux/live-bootstrap steps/bzip2-1.0.8/sources (musl rebuild entry)` beside the source pin;
- added `first-consumer: bootstrap/bzip2-1.0.8-musl.ncl` beside the source pin;
- fail-closed on `bzip2`, `bunzip2`, `bzcat`, and `bzip2recover` installed paths;
- added installed command help/usage probes for the four output commands.

## Verification

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/bzip2-1.0.8-musl.ncl \
  > openspec/changes/live-part-bzip2-1-0-8-musl/evidence/V1-source-pin-audit.log 2>&1
```

Exit status: 0

Transcript: `openspec/changes/live-part-bzip2-1-0-8-musl/evidence/V1-source-pin-audit.log`.

Key output:

```text
source-pin audit: 1 files, 1 fetch blocks, 0 issues
```
