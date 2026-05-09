# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.sed.4.0.9.musl

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/sed-4.0.9-musl.ncl > openspec/changes/live-part-sed-4-0-9-musl/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.
