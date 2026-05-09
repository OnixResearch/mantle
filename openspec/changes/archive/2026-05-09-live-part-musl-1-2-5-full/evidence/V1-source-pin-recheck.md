# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.musl.1.2.5.full

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/musl-full.ncl > openspec/changes/live-part-musl-1-2-5-full/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-musl-1-2-5-full/evidence/V1-source-pin-recheck.log`.
