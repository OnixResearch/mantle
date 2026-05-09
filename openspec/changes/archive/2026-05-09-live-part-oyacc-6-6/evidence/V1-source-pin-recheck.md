# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.oyacc.6.6

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/oyacc-tcc.ncl > openspec/changes/live-part-oyacc-6-6/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-oyacc-6-6/evidence/V1-source-pin-recheck.log`.
