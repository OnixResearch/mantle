# V1 source pin recheck evidence

Task-ID: V1
Covers: bootstrap.part.perl.5.000

Command:

```sh
cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/perl-5.000-musl.ncl > openspec/changes/live-part-perl-5-000/evidence/V1-source-pin-recheck.log 2>&1
```

Result: pass, exit code 0.

Transcript: `openspec/changes/live-part-perl-5-000/evidence/V1-source-pin-recheck.log`.
