# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.mpfr.4.1.0

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/mpfr-4.1.0.ncl > openspec/changes/live-part-mpfr-4-1-0/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-mpfr-4-1-0/evidence/V1-source-pin-recheck.log`.
