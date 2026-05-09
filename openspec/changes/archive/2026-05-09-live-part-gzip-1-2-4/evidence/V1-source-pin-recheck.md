# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.gzip.1.2.4

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/gzip-tcc.ncl > openspec/changes/live-part-gzip-1-2-4/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-gzip-1-2-4/evidence/V1-source-pin-recheck.log`.
