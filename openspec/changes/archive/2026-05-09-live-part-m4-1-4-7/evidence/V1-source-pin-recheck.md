# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.m4.1.4.7

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/m4-1.4.7-musl.ncl > openspec/changes/live-part-m4-1-4-7/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-m4-1-4-7/evidence/V1-source-pin-recheck.log`.
