# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.heirloom.devtools

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/heirloom-devtools.ncl > openspec/changes/live-part-heirloom-devtools/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-heirloom-devtools/evidence/V1-source-pin-recheck.log`.
