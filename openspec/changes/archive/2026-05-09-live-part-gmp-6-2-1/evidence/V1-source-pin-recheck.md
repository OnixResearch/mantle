# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.gmp.6.2.1

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/gmp-6.2.1.ncl > openspec/changes/live-part-gmp-6-2-1/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-gmp-6-2-1/evidence/V1-source-pin-recheck.log`.
