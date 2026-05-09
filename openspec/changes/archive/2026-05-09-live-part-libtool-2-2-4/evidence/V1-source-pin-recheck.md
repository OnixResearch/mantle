# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.libtool.2.2.4

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/libtool-2.2.4.ncl > openspec/changes/live-part-libtool-2-2-4/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-libtool-2-2-4/evidence/V1-source-pin-recheck.log`.
