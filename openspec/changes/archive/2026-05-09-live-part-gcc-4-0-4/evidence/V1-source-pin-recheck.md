# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.gcc.4.0.4

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/gcc-4.0.ncl > openspec/changes/live-part-gcc-4-0-4/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-gcc-4-0-4/evidence/V1-source-pin-recheck.log`.
