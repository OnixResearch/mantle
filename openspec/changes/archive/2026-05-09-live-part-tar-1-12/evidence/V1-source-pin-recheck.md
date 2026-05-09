# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.tar.1.12

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tar-tcc.ncl > openspec/changes/live-part-tar-1-12/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.
