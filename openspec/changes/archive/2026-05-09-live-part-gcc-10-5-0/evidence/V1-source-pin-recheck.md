# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.gcc.10.5.0

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/gcc-10.ncl > openspec/changes/live-part-gcc-10-5-0/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-gcc-10-5-0/evidence/V1-source-pin-recheck.log`.
