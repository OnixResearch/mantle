# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.gawk.3.0.4

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/gawk-3.0.4-musl.ncl > openspec/changes/live-part-gawk-3-0-4/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-gawk-3-0-4/evidence/V1-source-pin-recheck.log`.
